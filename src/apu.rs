// APU resources:
//
// Pan Doc:
// http://bgb.bircd.org/pandocs.htm#soundoverview
//
// Game Boy Sound Operation by Blarrg:
// https://gist.github.com/drhelius/3652407

use cpal::Sample;
use ringbuf::{Producer, RingBuffer};

use crate::{
    mmu::{
        NR10_REG, NR11_REG, NR12_REG, NR13_REG, NR14_REG, NR41_REG, NR42_REG, NR43_REG, NR44_REG,
        NR50_REG, NR51_REG, NR52_REG,
    },
    CLOCK_SPEED,
};

const TMP_SAMPLE_FREQ: u32 = CLOCK_SPEED as u32 / 4;

pub struct SquareWaveSoundGenerator {
    // Internal enabled flag.
    enabled: bool,
    duty: u8,
    _period: u32,
    _ctr: u32,
    _volume: i16,
    sample_count: u32,
    nr10: u8,
    _nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
    envelope: i16,
    envelope_step: u8,

    // The frame sequencer increments at 512 Hz and can be used
    // to derive all required low frequency clocks required
    frame_sequencer: u16,

    // Length counter. When this reaches zero the channel is disabled.
    length_counter: u8,

    // Length counter enabled (NRx4, bit 6)
    counter_enabled: bool,
}

pub struct NoiseGenerator {
    _sample_rate: u32,

    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,

    // Length counter. When this reaches zero the channel is disabled.
    length_counter: u8,
}

impl NoiseGenerator {
    #[allow(dead_code)]
    pub fn new(sample_rate: u32) -> Self {
        NoiseGenerator {
            _sample_rate: sample_rate,
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            length_counter: 0,
        }
    }

    #[allow(dead_code)]
    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            NR41_REG => self.nr41,
            NR42_REG => self.nr42,
            NR43_REG => self.nr43,
            NR44_REG => self.nr44,
            _ => 0,
        }
    }

    #[allow(dead_code)]
    pub fn write_reg(&mut self, address: u16, value: u8) {
        // println!("S1 write NR10 + {:02X} = {:02X}", address, value);
        match address {
            0 => {
                self.nr41 = value;
                self.length_counter = 64 - (value & 63);
            }

            1 => self.nr42 = value,
            2 => self.nr43 = value,
            3 => self.nr44 = value,

            _ => {}
        }
    }
}

impl SquareWaveSoundGenerator {
    pub fn new() -> Self {
        SquareWaveSoundGenerator {
            enabled: false,
            duty: 2,
            _period: 0,
            _ctr: 0,
            _volume: 16384,
            sample_count: 0,
            nr10: 0,
            _nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            envelope: 0,
            envelope_step: 0,
            frame_sequencer: 0,
            length_counter: 0,
            counter_enabled: false,
        }
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            NR10_REG => self.nr10,
            NR11_REG => 0xFF, // FIXME: bit 7 should be readable
            NR12_REG => self.nr12,
            NR13_REG => self.nr13,
            NR14_REG => self.nr14,
            _ => 0,
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8) {
        // println!("S1 write NR10 + {:02X} = {:02X}", address, value);
        match address {
            0 => self.nr10 = value, // FIXME!
            1 => {
                self.length_counter = 64 - (value & 63);
                self.duty = (value as u8 >> 6) & 3;
            }
            2 => self.nr12 = value,
            3 => self.nr13 = value,
            4 => {
                self.nr14 = value;
                self.counter_enabled = (value & 0x40) != 0;
                if value & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => {}
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

        self.envelope = ((self.nr12 >> 4) & 0xF) as i16;
        self.envelope_step = 0;
    }

    pub fn update(&mut self) -> i16 {
        let mut buf = Vec::new();

        let freq_raw: u32 = ((self.nr13 as u16) | (((self.nr14 & 0x07) as u16) << 8)) as u32;
        let freq = 131072 / (2048 - freq_raw);
        let freq_samples = TMP_SAMPLE_FREQ / freq;

        // When envelope steps is non-zero, the envelope (the amplitude)
        // will increase or decrease every (envelope_steps/64) second.
        let envelope_steps = self.nr12 & 7;

        if self.nr10 != 0 {
            println!("NOT IMPLEMENTED: sweep");
        }

        let mut tick_256hz = false;
        let mut tick_64hz = false;

        // Increment frame sequencer at 512 Hz
        if self.sample_count % (TMP_SAMPLE_FREQ / 512) == 0 {
            self.frame_sequencer = self.frame_sequencer.wrapping_add(1);
            tick_256hz = self.frame_sequencer & 1 == 0;
            tick_64hz = self.frame_sequencer & 7 == 0;
        }

        // Length counter. When the length counter decrements to zero
        // the channel gets disabled. It decrements at 256 Hz.
        if self.counter_enabled {
            if self.length_counter > 0 && tick_256hz {
                self.length_counter -= 1;
                if self.length_counter == 0 {
                    self.enabled = false;
                }
            }
        }

        // Envelope
        if tick_64hz {
            self.envelope_step += 1;
            if envelope_steps == 0 {
                self.envelope_step = 0;
            } else if self.envelope_step == envelope_steps {
                if self.nr12 & 8 == 0 {
                    if self.envelope > 0 {
                        self.envelope -= 1;
                    }
                } else {
                    if self.envelope < 0xf {
                        self.envelope += 1;
                    }
                }
                self.envelope_step = 0;
            }
        }

        let mut amplitude: i16 = 0;

        if self.enabled {
            if self.sample_count % freq_samples < (freq_samples / 2) {
                amplitude = self.envelope * 200;
            } else {
                amplitude = -self.envelope * 200;
            }
        }

        buf.push(amplitude);
        self.sample_count = self.sample_count.wrapping_add(1);
        return amplitude;
    }
}

pub struct AudioProcessingUnit {
    pub s1: SquareWaveSoundGenerator,
    pub s2: SquareWaveSoundGenerator,
    pub nr50: u8,
    pub nr51: u8,
    pub nr52: u8,

    // Producer for the output ring buffer.
    // Every cycle one sample is appended to this buffer.
    pub buf: Option<Producer<i16>>,
}

impl AudioProcessingUnit {
    pub fn new() -> Self {
        AudioProcessingUnit {
            s1: SquareWaveSoundGenerator::new(),
            s2: SquareWaveSoundGenerator::new(),
            nr50: 0,
            nr51: 0,
            nr52: 0,
            buf: None,
        }
    }

    pub fn update(&mut self) {
        if let Some(ref mut producer) = self.buf {
            if producer.is_full() {
                // eprintln!("Buffer is full");
                return;
            }

            if self.nr52 & 0x80 == 0 {
                producer
                    .push(0)
                    .expect("Failed to push sample to audio buffer");
                return;
            }

            let mut sample: i16 = 0;

            if self.nr52 & 0x01 == 0 {
                if self.nr51 & (1 | 16) != 0 {
                    sample += self.s1.update();
                }
            }

            if self.nr52 & 0x02 == 0 {
                if self.nr51 & (2 | 32) != 0 {
                    sample += self.s2.update();
                }
            }

            producer
                .push(sample)
                .expect("Failed to push sample to audio buffer");
        }
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            0xFF10..=0xFF14 => self.s1.read_reg(address),
            0xFF15..=0xFF19 => self.s2.read_reg(address),
            NR50_REG => self.nr50,
            NR51_REG => self.nr51,
            NR52_REG => self.nr52,
            _ => 0,
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8) {
        // println!("Write audio register 0x{:04X}: 0x{:02X}", address, value);
        match address {
            0xFF10..=0xFF14 => self.s1.write_reg(address - 0xFF10, value),
            0xFF15..=0xFF19 => self.s2.write_reg(address - 0xFF15, value),
            // 0xFF20..=0xFF23 => self.noise.write_reg(address - 0xFF20, value),
            NR50_REG => {
                // println!("NRF50 = {:02X}", value);
                self.nr50 = value
            }
            NR51_REG => {
                // println!("NRF51 = {:02X}", value);
                self.nr51 = value
            }
            NR52_REG => {
                // println!("NRF52 = {:02X}", value);
                self.nr52 = value
            }
            _ => {}
        }
    }
}
