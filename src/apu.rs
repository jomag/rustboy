// APU resources:
//
// Pan Doc:
// http://bgb.bircd.org/pandocs.htm#soundoverview
//
// Game Boy Sound Operation by Blargg:
// https://gist.github.com/drhelius/3652407
//
// GB Sound Emulation by Nightshade:
// https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html
//

// TODO:
// - For DMG hardware only, the length counters should be usable even
//   when NR52 is powered off. See the Blargg doc above.
// - After the sound hardware is powered on, frame sequencer should be
//   reset so next step is step 0.
// - Need to differentiate between a channel being "enabled" and the
//   DAC being enabled?
// - Remove duplicated envelope code

use ringbuf::Producer;

use crate::mmu::{NR50_REG, NR51_REG, NR52_REG};

// All channels have a length counter which counts down and disables
// the channel when it reaches zero. The length counter can be
// disabled.
pub struct LengthCounter {
    pub enabled: bool,
    pub value: u16,
}

impl LengthCounter {
    fn new() -> Self {
        LengthCounter {
            enabled: false,
            value: 0,
        }
    }

    // When triggered, if length counter is 0 it should
    // be reset to 64 (256 for wave channel).
    fn trigger(&mut self, reset_value: u16) {
        if self.value == 0 {
            self.value = reset_value;
        }
    }

    // This function should be called by the 256 Hz frame sequencer
    // tick. If it returns true, it has reached zero and the channel
    // should be disabled.
    fn count_down(&mut self) -> bool {
        if self.enabled {
            match self.value {
                0 => return false,
                1 => {
                    self.value = 0;
                    return true;
                }
                _ => {
                    self.value -= 1;
                    return false;
                }
            }
        } else {
            return false;
        }
    }
}

// Every channel has a DAC: a 4-bit digital-to-analog converter
// that generates a voltage from -1 to +1 for values 0 to 15.
pub struct DAC {
    powered_on: bool,
}

impl DAC {
    fn new() -> Self {
        DAC { powered_on: false }
    }

    fn convert(&self, inp: u8) -> f32 {
        assert!(inp & 0xF0 == 0);
        match self.powered_on {
            true => inp as f32 * (2.0 / 15.0) - 1.0,
            false => 0.0,
        }
    }
}

// SquareWaveSoundGenerator
// ------------------------
//
// Note that this type is used for both sound channel 1 and 2.
// The only difference is that channel 2 does not have any sweep
// generator and the registers starts at NR20 instead of NR10.
// To differentiate between them, there's the `with_sweep` boolean.
//
// ---------
// Registers
// ---------
//
// NR10 (0xFF10): Sweep. Only for sound channel 1.
// - bit 6..4: sweep time
// - bit 3:    sweep direction
// - bit 2..0: number of sweep shifts
//
// NR11 (0xFF11), NR21 (0xFF16): Wave pattern and sound length
// - bit 7..6: wave pattern, aka "duty".
// - bit 5..0: sound length (write only)
//
// NR12 (0xFF12), NR22 (0xFF17): Envelope
// - bit 7..4: initial volume
// - bit 3:    envelope direction
// - bit 2..0: number of envelope sweeps
//
// NR13 (0xFF13), NR23 (0xFF18): lo bits of frequency
// - bit 7..0: lo bits of frequency (write only)
//
// NR14 (0xFF14), NR24 (0xFF19): hi bits of frequency + more
// - bit 7: initial, 1 = restart sound (write only)
// - bit 6: length counter/consecutive selection
// - bit 2..0: hi bits of frequency (write only)
//
pub struct SquareWaveSoundGenerator {
    // Frequency. 10 bits. Bit 7..0 in NR13 + bit 9..8 in NR14.
    frequency: u16,

    // Internal register. When this counter reaches zero,
    // it is reset to the frequency value (NR13, NR14) and
    // wave_duty_position moves to next position
    frequency_timer: u16,

    // Duty Cycle Pattern. Bit 7..6 of NR11. R/W.
    duty: usize,

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

    // Initial volume of the envelope filter. Bit 7..4 in NR12
    initial_volume: u8,

    // Direction of the envelope. Bit 3 in NR12.
    // 0 (false) = decreasing, 1 (true) = increasing.
    envelope_increasing: bool,

    // Period count to initate envelope filter with on trigger.
    // Bits 2..0 of NR12.
    // FIXME: should there *be* a separate start-value,
    // or does the value count down and must be reset
    // before each trigger?
    envelope_periods_initial: u8,

    // Current volume of the envelope filter. Internal register.
    pub envelope: u8,

    // The envelope change period counter. Internal register.
    envelope_period: u8,

    // Internal enabled flag.
    pub enabled: bool,

    // Sweep time. Bit 6..4 of NR10. DOCUMENT ME!
    sweep_time: u8,

    // Sweep direction. Bit 3 of NR10. DOCUMENT ME!
    sweep_direction: bool,

    // Number of sweep shifts. Bit 2..0 of NR10. DOCUMENT ME!
    sweep_shift_count: u8,

    // Sweep is only enabled for the first square wave channel
    pub with_sweep: bool,

    pub length_counter: LengthCounter,
    pub dac: DAC,
}

const WAVE_DUTY: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

impl SquareWaveSoundGenerator {
    pub fn new(with_sweep: bool) -> Self {
        SquareWaveSoundGenerator {
            enabled: false,
            envelope: 0,
            envelope_period: 0,
            envelope_periods_initial: 0,
            envelope_increasing: false,
            initial_volume: 0,
            length_counter: LengthCounter::new(),
            frequency: 0,
            frequency_timer: 0,
            duty: 0,
            wave_duty_position: 0,
            sweep_time: 0,
            sweep_shift_count: 0,
            sweep_direction: false,
            with_sweep,
            dac: DAC::new(),
        }
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            0 => match self.with_sweep {
                true => {
                    let v =
                        ((self.sweep_time & 0b111) << 4) | (self.sweep_shift_count & 0b111) | 0x80;
                    if self.sweep_direction {
                        v | 0b1000
                    } else {
                        v
                    }
                }
                false => 0xFF,
            },
            1 => ((self.duty as u8) << 6) | 0b0011_1111,
            2 => {
                let v = (self.initial_volume << 4) | (self.envelope_periods_initial & 0b111);
                if self.envelope_increasing {
                    v | 0b1000
                } else {
                    v
                }
            }
            3 => 0xFF,
            4 => {
                if self.length_counter.enabled {
                    0xFF
                } else {
                    0b1011_1111
                }
            }
            _ => panic!("invalid register {}", address),
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8) {
        match address {
            0 => {
                self.sweep_time = (value >> 4) & 0b111;
                self.sweep_direction = (value & 0b1000) != 0;
                self.sweep_shift_count = value & 0b111;
            }
            1 => {
                self.duty = ((value >> 6) & 3) as usize;
                self.length_counter.value = (64 - (value & 63)) as u16;
            }
            2 => {
                self.initial_volume = (value >> 4) & 0xF;
                self.dac.powered_on = value & 0b1111_1000 != 0;
                self.enabled = self.enabled && self.dac.powered_on;
                self.envelope_increasing = (value & 0b1000) != 0;
                self.envelope_periods_initial = value & 0b111;
            }
            3 => self.frequency = (self.frequency & 0b11_0000_0000) | value as u16,
            4 => {
                self.frequency =
                    (self.frequency & 0b00_1111_1111) | (((value & 0b111) as u16) << 8);
                self.length_counter.enabled = value & 0b0100_0000 != 0;
                if value & 0b1000_0000 != 0 {
                    self.trigger();
                }
            }
            _ => panic!("invalid register {}", address),
        }
    }

    fn trigger(&mut self) {
        self.enabled = true;
        self.length_counter.trigger(64);
        self.frequency_timer = (2048 - self.frequency) * 4;
        self.envelope_period = self.envelope_periods_initial;
        self.envelope = self.initial_volume;

        // FIXME: add sweep trigger handling

        // If DAC is not powered, immediately disable the channel again
        if !self.dac.powered_on {
            self.enabled = false
        }
    }

    pub fn update(&mut self, hz64: bool, hz256: bool) -> f32 {
        // Decrement frequency timer
        if self.frequency_timer >= 4 {
            self.frequency_timer -= 4;

            // If frequency timer reaches 0, reset it to the selected frequency
            // (NR13, NR14) and increment the wave duty position
            if self.frequency_timer == 0 {
                self.frequency_timer = (2048 - self.frequency) * 4;
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
        let out = WAVE_DUTY[self.duty][self.wave_duty_position as usize];

        // if self.nr10 != 0 {
        // FIXME:
        // println!("NOT IMPLEMENTED: sweep");
        // }

        // Update length counter at 256 Hz
        if hz256 && self.length_counter.count_down() {
            self.enabled = false;
        }

        // When envelope steps is non-zero, the envelope (the amplitude)
        // will increase or decrease every (envelope_steps/64) second.

        // Envelope
        if hz64 && self.envelope_periods_initial > 0 {
            if self.envelope_period > 0 {
                self.envelope_period -= 1;

                if self.envelope_period == 0 {
                    self.envelope_period = self.envelope_periods_initial;

                    // Not max volume and volume should increase
                    if self.envelope < 0xF && self.envelope_increasing {
                        self.envelope += 1;
                    }

                    // Not min volume and volume should decrease
                    if self.envelope > 0 && !self.envelope_increasing {
                        self.envelope -= 1;
                    }
                }
            }
        }

        if self.enabled {
            let dac_input = out * self.envelope;
            return self.dac.convert(dac_input);
        }

        0.0
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

    // NR31 (0xFF1B): length load
    // 7..0: load sound length (write only)
    nr31: u8,

    // NR32 (0xFF1C): Volume envelope
    // bit 7, 4..0: not used
    // bit 6..5: volume code (0=0%, 1=100%, 2=50%, 3=25%)
    // nr32: u8,

    // NR33 (0xFF1D): lo bits of frequency
    // - bit 7..0: lo bits of frequency (write only)

    // NR34 (0xFF1E): hi bits of frequency + more
    // - bit 7:    trigger (write only)
    // - bit 6:    length counter enable
    // - bit 5..3: not used
    // - bit 2..0: hi bits of frequency (write only)

    // ---------------
    // Internal values
    // ---------------

    // Frequency. 10 bits. Bit 7..0 in NR13 + bit 9..8 in NR14.
    frequency: u16,

    // Wave pattern containing 32 4-bit samples.
    // These are accessed through 16 registers: 0xFF30 - 0xFF3F.
    // Each register holds two 4-bit samples. The upper bits are
    // played first.
    pub wave: [u8; CH3_WAVE_LENGTH],

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

    pub length_counter: LengthCounter,
    pub dac: DAC,
}

impl WaveSoundGenerator {
    pub fn new() -> Self {
        WaveSoundGenerator {
            nr31: 0,
            frequency: 0,
            wave: [0; CH3_WAVE_LENGTH],
            length_counter: LengthCounter::new(),
            wave_position: 0,
            frequency_timer: 0,
            enabled: false,
            volume_code: 0,
            dac: DAC::new(),
        }
    }

    // Reset everything except wave, which is what happens
    // when the sound hardware is powered off by NR52.
    pub fn power_off_reset(&mut self) {
        self.nr31 = 0;
        self.frequency = 0;
        self.length_counter = LengthCounter::new();
        self.wave_position = 0;
        self.frequency_timer = 0;
        self.enabled = false;
        self.volume_code = 0;
        self.dac = DAC::new();
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            0 => {
                if self.dac.powered_on {
                    0b1111_1111
                } else {
                    0b0111_1111
                }
            }

            1 => 0xFF,
            2 => self.volume_code << 5 | 0b1001_1111,
            3 => 0xFF,
            4 => {
                if self.length_counter.enabled {
                    0xFF
                } else {
                    0b1011_1111
                }
            }
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
                self.dac.powered_on = value & 0x80 != 0;
                self.enabled = self.enabled && self.dac.powered_on;
            }
            1 => {
                self.nr31 = value;
                self.length_counter.value = 256 - value as u16;
            }
            2 => self.volume_code = (value & 0b0110_0000) >> 5,
            3 => self.frequency = (self.frequency & 0b11_0000_0000) | value as u16,
            4 => {
                self.frequency =
                    (self.frequency & 0b00_1111_1111) | (((value & 0b111) as u16) << 8);
                self.length_counter.enabled = value & 0b0100_0000 != 0;
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
        self.length_counter.trigger(256);
        self.frequency_timer = (2048 - self.frequency) * 2;
        self.wave_position = 0;

        // If DAC is not powered on, immediately disable the channel again
        if !self.dac.powered_on {
            self.enabled = false
        }
    }

    pub fn update(&mut self, hz256: bool) -> f32 {
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
                self.frequency_timer = (2048 - self.frequency) * 2;
                self.wave_position = (self.wave_position + 1) & 31;
            }
        } else {
            self.frequency_timer -= 4;
        }

        let mut out = self.wave[self.wave_position as usize];

        // Update length counter at 256 Hz
        if hz256 && self.length_counter.count_down() {
            self.enabled = false;
        }

        out = match self.volume_code {
            0 => 0,
            1 => out,
            2 => out >> 1,
            3 => out >> 2,
            _ => 0,
        };

        if self.enabled {
            return self.dac.convert(out);
        }

        return 0.0;
    }
}

pub struct NoiseSoundGenerator {
    // ---------
    // Registers
    // ---------

    // NR41 (0xFF20): length load
    // 5..0: load sound length (write only)

    // NR42 (0xFF21): Volume envelope
    // bit 7..4: initial volume
    // bit 3:    envelope direction
    // bit 2..0: sweep count

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

    // Current volume of the envelope filter. Internal register.
    pub envelope: u8,

    // The envelope change period counter. Internal register.
    envelope_period: u8,

    // Period count to initate envelope filter with on trigger.
    // Bits 2..0 of NR12.
    // FIXME: should there *be* a separate start-value,
    // or does the value count down and must be reset
    // before each trigger?
    envelope_periods_initial: u8,

    envelope_increasing: bool,

    // Initial volume of the envelope filter. Bit 7..4 in NR12
    initial_volume: u8,

    pub length_counter: LengthCounter,
    pub dac: DAC,
}

const NOISE_DIVISOR_MAP: [u8; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

impl NoiseSoundGenerator {
    pub fn new() -> Self {
        NoiseSoundGenerator {
            nr43: 0,
            nr44: 0,
            frequency_timer: 0,
            lfsr: 0,
            polynomial_counter: 0,
            enabled: false,
            envelope: 0,
            envelope_period: 0,
            envelope_increasing: false,
            envelope_periods_initial: 0,
            initial_volume: 0,
            length_counter: LengthCounter::new(),
            dac: DAC::new(),
        }
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            0 => 0xFF,
            1 => {
                let nr42 = (self.initial_volume << 4) | self.envelope_periods_initial;
                if self.envelope_increasing {
                    nr42 | 0b0000_1000
                } else {
                    nr42
                }
            }
            2 => self.nr43,
            3 => {
                if self.length_counter.enabled {
                    0b1111_1111
                } else {
                    0b1011_1111
                }
            }
            _ => panic!("invalid register {}", address),
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8) {
        match address {
            0 => self.length_counter.value = (64 - (value & 63)) as u16,
            1 => {
                self.initial_volume = (value >> 4) & 0xF;
                self.dac.powered_on = value & 0b1111_1000 != 0;
                self.enabled = self.enabled && self.dac.powered_on;
                self.envelope_increasing = (value & 0b1000) != 0;
                self.envelope_periods_initial = value & 0b111;
            }
            2 => self.nr43 = value,
            3 => {
                self.length_counter.enabled = value & 0b0100_0000 != 0;
                if value & 0b1000_0000 != 0 {
                    self.trigger();
                }
            }
            _ => panic!("invalid register {}", address),
        }
    }

    pub fn trigger(&mut self) {
        self.enabled = true;
        self.length_counter.trigger(64);
        self.lfsr = 0b0111_1111_1111_1111;

        let divisor_code = self.nr43 & 7;
        let divisor = NOISE_DIVISOR_MAP[divisor_code as usize];
        let shift_amount = (self.nr43 & 0xF0) >> 4;
        self.frequency_timer = (divisor as u16) << (shift_amount as u16);

        self.envelope = self.initial_volume;
        self.envelope_period = self.envelope_periods_initial;

        // If DAC is not powered, immediately disable the channel again
        if !self.dac.powered_on {
            self.enabled = false
        }
    }

    pub fn update(&mut self, hz64: bool, hz256: bool) -> f32 {
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

        // Update length counter at 256 Hz
        if hz256 && self.length_counter.count_down() {
            self.enabled = false;
        }

        // When envelope steps is non-zero, the envelope (the amplitude)
        // will increase or decrease every (envelope_steps/64) second.

        // Envelope
        if hz64 && self.envelope_periods_initial > 0 {
            if self.envelope_period > 0 {
                self.envelope_period -= 1;

                if self.envelope_period == 0 {
                    self.envelope_period = self.envelope_periods_initial;

                    // Not max volume and volume should increase
                    if self.envelope < 0xF && self.envelope_increasing {
                        self.envelope += 1;
                    }

                    // Not min volume and volume should decrease
                    if self.envelope > 0 && !self.envelope_increasing {
                        self.envelope -= 1;
                    }
                }
            }
        }

        if self.enabled {
            let out = if self.lfsr & 1 == 0 { 0 } else { 1 };
            let dac_input = out * self.envelope;
            return self.dac.convert(dac_input);
        }

        0.0
    }
}

pub trait AudioRecorder {
    fn mono(&mut self, sample: f32);
    fn gen1(&mut self, sample: f32);
    fn gen2(&mut self, sample: f32);
    fn flush(&mut self);
}

pub struct AudioProcessingUnit {
    pub s1: SquareWaveSoundGenerator,
    pub s2: SquareWaveSoundGenerator,
    pub ch3: WaveSoundGenerator,
    pub ch4: NoiseSoundGenerator,
    pub nr50: u8,
    pub nr51: u8,

    // Bit 7 of NR52. Controls power to the audio hardware
    pub powered_on: bool,

    // Producer for the output ring buffer.
    // Every cycle one sample is appended to this buffer.
    pub buf: Option<Producer<f32>>,

    pub recorder: Option<Box<dyn AudioRecorder>>,
}

impl AudioProcessingUnit {
    pub fn new() -> Self {
        AudioProcessingUnit {
            s1: SquareWaveSoundGenerator::new(true),
            s2: SquareWaveSoundGenerator::new(false),
            ch3: WaveSoundGenerator::new(),
            ch4: NoiseSoundGenerator::new(),
            nr50: 0,
            nr51: 0,
            buf: None,
            recorder: None,
            powered_on: false,
        }
    }

    // Perform a complete reset. Used when resetting the whole machine.
    // Note that the APU can't easily be recreated, as it has a ringbuf
    // producer that can't be moved to a new instance of it, so instead
    // we must reset all values.
    pub fn reset(&mut self) {
        self.s1 = SquareWaveSoundGenerator::new(true);
        self.s2 = SquareWaveSoundGenerator::new(false);
        self.ch3 = WaveSoundGenerator::new();
        self.ch4 = NoiseSoundGenerator::new();
        self.nr50 = 0;
        self.nr51 = 0;
        self.powered_on = false;
    }

    pub fn update(&mut self, div_counter: u16) {
        // NR52 bit 7 is used to disable the sound system completely
        if !self.powered_on {
            if let Some(ref mut prod) = self.buf {
                prod.push(0.0)
                    .expect("Failed to push sample to audio buffer");
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

        let mut ch1_output: f32 = 0.0;
        let mut ch2_output: f32 = 0.0;
        let mut ch3_output: f32 = 0.0;
        let mut ch4_output: f32 = 0.0;

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

        let sample = (ch1_output + ch2_output + ch3_output + ch4_output) as f32;

        if let Some(ref mut rec) = self.recorder {
            rec.gen1(ch1_output as f32);
            rec.gen2(ch2_output as f32);
            rec.mono(sample);
        }

        if let Some(ref mut producer) = self.buf {
            if producer.is_full() {
                eprintln!("Buffer is full {} {}", producer.capacity(), producer.len());
                return;
            }

            producer
                .push(sample as f32)
                .expect("Failed to push sample to audio buffer");
        }
    }

    pub fn read_nr52(&self) -> u8 {
        let mut nr52: u8 = 0;
        if self.powered_on {
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
            0xFF1F => 0xFF,
            0xFF20..=0xFF23 => self.ch4.read_reg(address - 0xFF20),
            NR50_REG => self.nr50,
            NR51_REG => self.nr51,
            NR52_REG => self.read_nr52() | 0b0111_0000,
            0xFF27..=0xFF2F => 0xFF,
            0xFF30..=0xFF3F => self.ch3.read_wave_reg(address as usize - 0xFF30),
            _ => 0,
        }
    }

    fn power_on(&mut self) {
        if !self.powered_on {
            self.powered_on = true;
        }
    }

    fn power_off(&mut self) {
        if self.powered_on {
            self.powered_on = false;
            self.s1 = SquareWaveSoundGenerator::new(true);
            self.s2 = SquareWaveSoundGenerator::new(false);
            self.ch3.power_off_reset();
            self.ch4 = NoiseSoundGenerator::new();
            self.nr50 = 0;
            self.nr51 = 0;
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8) {
        // Writes to NR52 and the wave memory allways work, even
        // when the sound hardware is powered off.
        match address {
            NR52_REG => {
                if value & 0x80 != 0 {
                    self.power_on();
                } else {
                    self.power_off();
                }
            }
            0xFF30..=0xFF3F => self.ch3.write_wave_reg(address as usize - 0xFF30, value),
            _ => {}
        }

        if self.powered_on {
            match address {
                0xFF10..=0xFF14 => self.s1.write_reg(address - 0xFF10, value),
                0xFF15..=0xFF19 => self.s2.write_reg(address - 0xFF15, value),
                0xFF1A..=0xFF1E => self.ch3.write_reg(address - 0xFF1A, value),
                0xFF1F => {}
                0xFF20..=0xFF23 => self.ch4.write_reg(address - 0xFF20, value),
                NR50_REG => self.nr50 = value,
                NR51_REG => self.nr51 = value,
                0xFF27..=0xFF2F => {}
                _ => {}
            }
        }
    }
}

fn reg_name(address: u16) -> String {
    match address {
        0xFF10..=0xFF14 => format!("NR{:02X}", address - 0xFF10 + 0x10),
        0xFF15..=0xFF19 => format!("NR{:02X}", address - 0xFF15 + 0x20),
        0xFF1A..=0xFF1E => format!("NR{:02X}", address - 0xFF1A + 0x30),
        0xFF1F..=0xFF23 => format!("NR{:02X}", address - 0xFF1F + 0x40),
        _ => format!("{:04X}", address),
    }
}
