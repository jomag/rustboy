// APU resources:
//
// Pan Doc:
// http://bgb.bircd.org/pandocs.htm#soundoverview
//
// Game Boy Sound Operation by Blarrg:
// https://gist.github.com/drhelius/3652407
//
// GB Sound Emulation by Nightshade:
// https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html

use ringbuf::Producer;

use crate::mmu::{NR50_REG, NR51_REG, NR52_REG};

// Note that this type is used for both sound channel 1 and 2.
// The only difference is that channel 2 does not have any sweep
// generator and the registers starts at NR20 instead of NR10.
pub struct SquareWaveSoundGenerator {
    // ---------
    // Registers
    // ---------

    // NR10 (0xFF10): Sweep. Only for sound channel 1.
    // - bit 6..4: sweep time
    // - bit 3:    sweep direction
    // - bit 2..0: number of sweep shifts
    nr10: u8,

    // NR11 (0xFF11), NR21 (0xFF16): Wave pattern and sound length
    // - bit 7..6: wave pattern
    // - bit 5..0: sound length (write only)
    nr11: u8,

    // NR12 (0xFF12), NR22 (0xFF17): Envelope
    // - bit 7..4: initial volume
    // - bit 3:    envelope direction
    // - bit 2..0: number of envelope sweeps
    nr12: u8,

    // NR13 (0xFF13), NR23 (0xFF18): lo bits of frequency
    // - bit 7..0: lo bits of frequency (write only)
    nr13: u8,

    // NR14 (0xFF14), NR24 (0xFF19): hi bits of frequency + more
    // - bit 7: initial, 1 = restart sound (write only)
    // - bit 6: length counter/consecutive selection
    // - bit 2..0: hi bits of frequency (write only)
    nr14: u8,

    // Internal register. When this counter reaches zero,
    // it is reset to the frequency value (NR13, NR14) and
    // wave_duty_position moves to next position
    frequency_timer: u16,

    // Internal register. Holds a value between 0 and 7
    // and decides if the current output of the square wave
    // should be high or low. One of the following patterns
    // are selected with NR11 and the output is high if
    // the corresponding bit is high:
    //
    // 0: 12.5% - 00000001
    // 1: 25%   - 00000011
    // 2: 50%   - 00001111
    // 3: 75%   - 00111111
    //
    wave_duty_position: u16,

    // Current volume of the envelope filter. Internal register.
    pub envelope: i16,

    // The envelope change period counter. Internal register.
    envelope_period: u8,

    // Length counter. Internal register.
    // When this reaches zero the channel is disabled.
    pub length_counter: u8,

    // Internal enabled flag.
    pub enabled: bool,
}

const WAVE_DUTY: [[i16; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

impl SquareWaveSoundGenerator {
    pub fn new() -> Self {
        SquareWaveSoundGenerator {
            enabled: false,
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            envelope: 0,
            envelope_period: 0,
            length_counter: 0,

            frequency_timer: 0,
            wave_duty_position: 0,
        }
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            0 => self.nr10,
            1 => self.nr11,
            2 => self.nr12,
            3 => self.nr13,
            4 => self.nr14,
            _ => panic!("invalid register {}", address),
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8) {
        // println!("S1 write NR10 + {:02X} = {:02X}", address, value);
        match address {
            0 => self.nr10 = value, // FIXME!
            1 => {
                self.nr11 = value;
                self.length_counter = 64 - (value & 63);
            }
            2 => self.nr12 = value,
            3 => self.nr13 = value,
            4 => {
                self.nr14 = value;
                if value & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => panic!("invalid register {}", address),
        }
    }

    fn trigger(&mut self) {
        // See details about exactly what happens on sound trigger
        // in the document Game Boy Sound Operation by Blarrg:
        // https://gist.github.com/drhelius/3652407
        self.enabled = true;

        if self.length_counter == 0 {
            self.length_counter = 64;
        }

        let frequency: u16 = ((self.nr13 as u16) | (((self.nr14 & 0x07) as u16) << 8)) as u16;
        self.frequency_timer = (2048 - frequency) * 4;

        self.envelope = ((self.nr12 >> 4) & 0xF) as i16;
        self.envelope_period = self.nr12 & 7;
    }

    pub fn update(&mut self, hz64: bool, hz256: bool) -> i16 {
        // Decrement frequency timer
        if self.frequency_timer >= 4 {
            self.frequency_timer -= 4;

            // If frequency timer reaches 0, reset it to the selected frequency
            // (NR13, NR14) and increment the wave duty position
            if self.frequency_timer == 0 {
                let frequency: u16 =
                    ((self.nr13 as u16) | (((self.nr14 & 0x07) as u16) << 8)) as u16;
                self.frequency_timer = (2048 - frequency) * 4;
                self.wave_duty_position = (self.wave_duty_position + 1) & 7;
            }
        }

        // There are four available duty patterns that sets for
        // what part of a period the wave should have high state.
        //
        // 0: 12.5% - 00000001
        // 1: 25%   - 00000011
        // 2: 50%   - 00001111
        // 3: 75%   - 00111111
        //
        // The duty pattern is stored in bit 6-7 of NR11 (NR21)
        let pattern = ((self.nr11 >> 6) & 3) as usize;
        let out = WAVE_DUTY[pattern][self.wave_duty_position as usize];

        if self.nr10 != 0 {
            // FIXME:
            // println!("NOT IMPLEMENTED: sweep");
        }

        // Length counter. When length counter is enabled (bit 6 of NRx4)
        // and there is a 256 Hz clock, the length counter decrements.
        // If it reaches zero the channel gets disabled.
        if hz256 && (self.nr14 & 0x40) != 0 && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }

        // When envelope steps is non-zero, the envelope (the amplitude)
        // will increase or decrease every (envelope_steps/64) second.
        let envelope_period = self.nr12 & 7;

        // Envelope
        if hz64 && envelope_period > 0 {
            if self.envelope_period > 0 {
                self.envelope_period -= 1;

                if self.envelope_period == 0 {
                    self.envelope_period = envelope_period;

                    // Not max volume and volume should increase
                    if self.envelope < 0xF && (self.nr12 & 8) != 0 {
                        self.envelope += 1;
                    }

                    // Not min volume and volume should decrease
                    if self.envelope > 0 && (self.nr12 & 8) == 0 {
                        self.envelope -= 1;
                    }
                }
            }
        }

        if self.enabled {
            let dac_input = out * self.envelope;
            return dac_input * 200 - 100;
        }

        return 0;
    }
}
pub const CH3_WAVE_LENGTH: usize = 32;

pub struct WaveSoundGenerator {
    // ---------
    // Registers
    // ---------

    // NR30 (0xFF1A): DAC power
    // bit 7:    dac power
    // bit 6..0: not used
    nr30: u8,

    // NR31 (0xFF1B): length load
    // 7..0: load sound length (write only)
    nr31: u8,

    // NR32 (0xFF1C): Volume envelope
    // bit 7, 4..0: not used
    // bit 6..5: volume code (0=0%, 1=100%, 2=50%, 3=25%)
    // nr32: u8,

    // NR33 (0xFF1D): lo bits of frequency
    // - bit 7..0: lo bits of frequency (write only)
    nr33: u8,

    // NR34 (0xFF1E): hi bits of frequency + more
    // - bit 7:    trigger (write only)
    // - bit 6:    length counter enable
    // - bit 5..3: not used
    // - bit 2..0: hi bits of frequency (write only)
    nr34: u8,

    // ---------------
    // Internal values
    // ---------------

    // Wave pattern containing 32 4-bit samples.
    // These are accessed through 16 registers: 0xFF30 - 0xFF3F.
    // Each register holds two 4-bit samples. The upper bits are
    // played first.
    pub wave: [u8; CH3_WAVE_LENGTH],

    // Length counter. Internal register.
    // When this reaches zero the channel is disabled.
    pub length_counter: u16,

    // Internal enabled flag.
    pub enabled: bool,

    // Internal register. When this counter reaches zero,
    // it is reset to the frequency value (NR13, NR14) and
    // wave_duty_position moves to next position
    pub frequency_timer: u16,

    pub wave_position: u16,

    // Volume code (0=0%, 1=100%, 2=50%, 3=25%)
    // Bits 6..5 of NR32
    pub volume_code: u8,
}

impl WaveSoundGenerator {
    pub fn new() -> Self {
        WaveSoundGenerator {
            nr30: 0,
            nr31: 0,
            nr33: 0,
            nr34: 0,
            wave: [0; CH3_WAVE_LENGTH],
            length_counter: 0,
            wave_position: 0,
            frequency_timer: 0,
            enabled: false,
            volume_code: 0,
        }
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            0 => self.nr30,
            1 => self.nr31,
            2 => self.volume_code << 5,
            3 => self.nr33,
            4 => self.nr34,
            _ => panic!("invalid register in channel 3: {}", address),
        }
    }

    pub fn read_wave_reg(&self, address: usize) -> u8 {
        match address {
            0..=0xF => (self.wave[address * 2] << 4) | self.wave[address * 2 + 1],
            _ => panic!("attempt to read wave pattern register {}", address),
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8) {
        match address {
            0 => {
                self.nr30 = value;
                if value & 0x80 == 0 {
                    self.enabled = false;
                }
            }
            1 => {
                self.nr31 = value;
                self.length_counter = 256 - value as u16;
            }
            2 => self.volume_code = (value & 0b0110_0000) >> 5,
            3 => self.nr33 = value,
            4 => {
                self.nr34 = value;
                if value & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => panic!("invalid register in channel 3: {}", address),
        }
    }

    pub fn write_wave_reg(&mut self, address: usize, value: u8) {
        self.wave[address * 2] = (value & 0xF0) >> 4;
        self.wave[address * 2 + 1] = value & 0x0F
    }

    fn trigger(&mut self) {
        self.enabled = true;

        if self.length_counter == 0 {
            self.length_counter = 256;
        }

        let frequency: u16 = ((self.nr33 as u16) | (((self.nr34 & 0x07) as u16) << 8)) as u16;
        self.frequency_timer = (2048 - frequency) * 2;

        self.wave_position = 0;
    }

    pub fn update(&mut self, hz256: bool) -> i16 {
        if self.nr30 & 0x80 == 0 {
            self.enabled = false;
        }

        // Decrement frequency timer
        // FIXME: this timer can end up at value 2, as the frequency is multiplied by 2.
        // This is problematic as the APU is updated on every 4'th cycle.
        // For the other channels this is not a problem, as it's always divisible by
        // 4. We must consider what side effects this can have. We should probably
        // compensate when the new frequency timer and wave position is selected below
        // depending on if it reached zero from 4 or 2.
        if self.frequency_timer < 4 {
            self.frequency_timer = 0;

            // If frequency timer reaches 0, reset it to the selected frequency
            // (NR13, NR14) and increment the wave position
            if self.frequency_timer == 0 {
                let frequency: u16 =
                    ((self.nr33 as u16) | (((self.nr34 & 0x07) as u16) << 8)) as u16;
                self.frequency_timer = (2048 - frequency) * 2;
                self.wave_position = (self.wave_position + 1) & 31;
            }
        } else {
            self.frequency_timer -= 4;
        }

        let mut out = self.wave[self.wave_position as usize];

        // Length counter. When length counter is enabled (bit 6 of NRx4)
        // and there is a 256 Hz clock, the length counter decrements.
        // If it reaches zero the channel gets disabled.
        if hz256 && (self.nr34 & 0x40) != 0 && self.length_counter > 0 {
            println!("Never happens?");
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }

        out = match self.volume_code {
            0 => 0,
            1 => out,
            2 => out >> 1,
            3 => out >> 2,
            _ => 0,
        };

        if self.enabled {
            return (out as i16) * 200 - 100;
        }

        return 0;
    }
}

pub struct NoiseSoundGenerator {
    // ---------
    // Registers
    // ---------

    // NR41 (0xFF20): length load
    // 5..0: load sound length (write only)
    nr41: u8,

    // NR42 (0xFF21): Volume envelope
    // bit 7..4: initial volume
    // bit 3:    envelope direction
    // bit 2..0: sweep count
    nr42: u8,

    // NR43 (0xFF22): polynomial counter
    // bit 7..4: shift clock frequency
    // bit 3:    counter step/width (0=15 bits, 1=7 bits)
    // bit 2..0: dividing ratio of frequencies
    nr43: u8,

    // NR44 (0xFF23): counter/consecutive, initial
    // bit 7: initial, 1=restart sound
    // bit 6: counter/consecutive selection (1=stop when length expires)
    nr44: u8,

    // Internal register. When this counter reaches zero,
    // it is reset to the frequency value.
    pub frequency_timer: u16,

    // LFSR register. Internal. 15 bits.
    pub lfsr: u16,

    // Polynomial counter. Internal.
    polynomial_counter: u8,

    // Internal register
    pub enabled: bool,

    // Internal register
    length_counter: u8,

    // Current volume of the envelope filter. Internal register.
    pub envelope: i16,

    // The envelope change period counter. Internal register.
    envelope_period: u8,
}

const NOISE_DIVISOR_MAP: [u8; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

impl NoiseSoundGenerator {
    pub fn new() -> Self {
        NoiseSoundGenerator {
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            frequency_timer: 0,
            lfsr: 0,
            polynomial_counter: 0,
            enabled: false,
            length_counter: 0,
            envelope: 0,
            envelope_period: 0,
        }
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            0 => self.nr41,
            1 => self.nr42,
            2 => self.nr43,
            3 => self.nr44,
            _ => panic!("invalid register {}", address),
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8) {
        match address {
            0 => self.nr41 = value,
            1 => {
                self.nr42 = value;
                self.length_counter = 64 - (value & 63);
            }
            2 => self.nr43 = value,
            3 => {
                self.nr44 = value;
                if value & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => panic!("invalid register {}", address),
        }
    }

    pub fn trigger(&mut self) {
        self.enabled = true;
        self.lfsr = 0b0111_1111_1111_1111;

        if self.length_counter == 0 {
            self.length_counter = 64;
        }

        let divisor_code = self.nr43 & 7;
        let divisor = NOISE_DIVISOR_MAP[divisor_code as usize];
        let shift_amount = (self.nr43 & 0xF0) >> 4;
        self.frequency_timer = (divisor as u16) << (shift_amount as u16);

        self.envelope = ((self.nr42 >> 4) & 0xF) as i16;
        self.envelope_period = self.nr42 & 7;
    }

    pub fn update(&mut self, hz64: bool, hz256: bool) -> i16 {
        // Decrement frequency timer
        if self.frequency_timer >= 4 {
            self.frequency_timer -= 4;

            if self.frequency_timer == 0 {
                let divisor_code = self.nr43 & 7;
                let divisor = NOISE_DIVISOR_MAP[divisor_code as usize];
                let shift_amount = (self.nr43 & 0xF0) >> 4;
                self.frequency_timer = (divisor as u16) << (shift_amount as u16);

                let xor_result = (self.lfsr & 1) ^ ((self.lfsr & 2) >> 1);
                self.lfsr = (self.lfsr >> 1) | (xor_result << 14);

                if self.nr43 & 0b1000 != 0 {
                    self.lfsr &= !(1 << 6);
                    self.lfsr |= xor_result << 6;
                }
            }
        }

        // Length counter. When length counter is enabled (bit 6 of NRx4)
        // and there is a 256 Hz clock, the length counter decrements.
        // If it reaches zero the channel gets disabled.
        if hz256 && (self.nr44 & 0x40) != 0 && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }

        // When envelope steps is non-zero, the envelope (the amplitude)
        // will increase or decrease every (envelope_steps/64) second.
        let envelope_period = self.nr42 & 7;

        // Envelope
        if hz64 && envelope_period > 0 {
            if self.envelope_period > 0 {
                self.envelope_period -= 1;

                if self.envelope_period == 0 {
                    self.envelope_period = envelope_period;

                    // Not max volume and volume should increase
                    if self.envelope < 0xF && (self.nr42 & 8) != 0 {
                        self.envelope += 1;
                    }

                    // Not min volume and volume should decrease
                    if self.envelope > 0 && (self.nr42 & 8) == 0 {
                        self.envelope -= 1;
                    }
                }
            }
        }

        if self.enabled {
            if self.enabled {
                let out = if self.lfsr & 1 == 0 { 0 } else { 1 };
                let dac_input = out * self.envelope;
                return dac_input * 200 - 100;
            }
        }

        0
    }
}

pub trait AudioRecorder {
    fn mono(&mut self, sample: i16);
    fn gen1(&mut self, sample: i16);
    fn gen2(&mut self, sample: i16);
    fn flush(&mut self);
}

pub struct AudioProcessingUnit {
    pub s1: SquareWaveSoundGenerator,
    pub s2: SquareWaveSoundGenerator,
    pub ch3: WaveSoundGenerator,
    pub ch4: NoiseSoundGenerator,
    pub nr50: u8,
    pub nr51: u8,

    // Bit 7 of NR52
    pub enabled: bool,

    // Producer for the output ring buffer.
    // Every cycle one sample is appended to this buffer.
    pub buf: Option<Producer<i16>>,

    pub recorder: Option<Box<dyn AudioRecorder>>,
}

impl AudioProcessingUnit {
    pub fn new() -> Self {
        AudioProcessingUnit {
            s1: SquareWaveSoundGenerator::new(),
            s2: SquareWaveSoundGenerator::new(),
            ch3: WaveSoundGenerator::new(),
            ch4: NoiseSoundGenerator::new(),
            nr50: 0,
            nr51: 0,
            buf: None,
            recorder: None,
            enabled: false,
        }
    }

    pub fn update(&mut self, div_counter: u16) {
        // NR52 bit 7 is used to disable the sound system completely
        if !self.enabled {
            if let Some(ref mut prod) = self.buf {
                prod.push(0).expect("Failed to push sample to audio buffer");
            }
            return;
        }

        // The Frame Sequencer is used to generate clocks as 256 Hz,
        // 128 Hz and 64 Hz. See this table copied from gbdev wiki:
        //
        // Step   Length Ctr  Vol Env     Sweep
        // ---------------------------------------
        // 0      Clock       -           -
        // 1      -           -           -
        // 2      Clock       -           Clock
        // 3      -           -           -
        // 4      Clock       -           -
        // 5      -           -           -
        // 6      Clock       -           Clock
        // 7      -           Clock       -
        // ---------------------------------------
        // Rate   256 Hz      64 Hz       128 Hz
        //
        // To allow for all these to be generated, the frame sequencer
        // must tick at 512 Hz (every 8192'th cycle). Since the pattern
        // repeats every 8'th time, we can initialize frame sequencer to
        // 65536 (8192 * 512), decrement to zero and wrap around.
        //
        // The frame sequencer is based on the DIV timer. DIV is the top
        // 8 bits of the 16-bit timer that decrements for every clock cycle.
        //
        // Note that for CGB, div_counter should be shifted 14 bits instead of 13
        // as the DIV registers decrements at double speed. That means only two
        // bits remain, so we must have another strategy for the 64 Hz clock.
        let mut hz64 = false;
        let mut hz128 = false;
        let mut hz256 = false;
        if div_counter % 8192 == 0 {
            let step = div_counter >> 13;
            hz64 = step == 7;
            hz128 = step == 2 || step == 6;
            hz256 = step & 1 == 0;
        }

        let mut ch1_output: i16 = 0;
        let mut ch2_output: i16 = 0;
        let mut ch3_output: i16 = 0;
        let mut ch4_output: i16 = 0;

        if self.nr51 & (1 | 16) != 0 {
            ch1_output = self.s1.update(hz64, hz256);
        }

        if self.nr51 & (2 | 32) != 0 {
            ch2_output = self.s2.update(hz64, hz256);
        }

        if self.nr51 & (4 | 64) != 0 {
            ch3_output = self.ch3.update(hz256);
        }

        if self.nr51 & (8 | 128) != 0 {
            ch4_output = self.ch4.update(hz64, hz256);
        }

        let sample = ch1_output + ch2_output + ch3_output + ch4_output;

        if let Some(ref mut rec) = self.recorder {
            rec.gen1(ch1_output);
            rec.gen2(ch2_output);
            rec.mono(sample);
        }

        if let Some(ref mut producer) = self.buf {
            if producer.is_full() {
                eprintln!("Buffer is full {} {}", producer.capacity(), producer.len());
                return;
            }

            producer
                .push(sample)
                .expect("Failed to push sample to audio buffer");
        }
    }

    pub fn read_nr52(&self) -> u8 {
        let mut nr52: u8 = 0;
        if self.enabled {
            nr52 = 0x80;
        }
        if self.s1.enabled {
            nr52 |= 0b0001;
        }
        if self.s2.enabled {
            nr52 |= 0b0010;
        }
        if self.ch3.enabled {
            nr52 |= 0b0100;
        }
        if self.ch4.enabled {
            nr52 |= 0b1000;
        }
        nr52
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            0xFF10..=0xFF14 => self.s1.read_reg(address - 0xFF10),
            0xFF15..=0xFF19 => self.s2.read_reg(address - 0xFF15),
            0xFF1A..=0xFF1E => self.ch3.read_reg(address - 0xFF1A),
            0xFF20..=0xFF23 => self.ch4.read_reg(address - 0xFF20),
            0xFF30..=0xFF3F => self.ch3.read_wave_reg(address as usize - 0xFF30),
            NR50_REG => self.nr50,
            NR51_REG => self.nr51,
            NR52_REG => self.read_nr52(),
            _ => 0,
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8) {
        // println!("Write audio register 0x{:04X}: 0x{:02X}", address, value);
        match address {
            0xFF10..=0xFF14 => self.s1.write_reg(address - 0xFF10, value),
            0xFF15..=0xFF19 => self.s2.write_reg(address - 0xFF15, value),
            0xFF1A..=0xFF1E => self.ch3.write_reg(address - 0xFF1A, value),
            0xFF20..=0xFF23 => self.ch4.write_reg(address - 0xFF20, value),
            0xFF30..=0xFF3F => self.ch3.write_wave_reg(address as usize - 0xFF30, value),
            NR50_REG => self.nr50 = value,
            NR51_REG => self.nr51 = value,

            // Note that for proper emulation, when sound is disabled:
            // - All sound registers gets destroyed
            // - It's not possible to access any sound registers except NR52
            NR52_REG => self.enabled = value & 0x80 != 0,

            _ => {}
        }
    }
}
