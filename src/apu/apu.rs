use crate::{
    apu::noise_gen::NoiseSoundGenerator,
    apu::square_gen::SquareWaveSoundGenerator,
    apu::wave_gen::WaveSoundGenerator,
    emu::Machine,
    mmu::{
        NR10_REG, NR11_REG, NR12_REG, NR13_REG, NR14_REG, NR20_REG, NR21_REG, NR22_REG, NR23_REG,
        NR24_REG, NR30_REG, NR31_REG, NR32_REG, NR33_REG, NR34_REG, NR40_REG, NR41_REG, NR42_REG,
        NR43_REG, NR44_REG, NR50_REG, NR51_REG, NR52_REG,
    },
    CYCLES_PER_FRAME,
};

use blip_buf::BlipBuf;
use num_traits::abs;

// Approx numberof samples per frame. The actual count is a little less than this.
pub const SAMPLES_PER_FRAME: usize = CYCLES_PER_FRAME / 59;

pub trait AudioRecorder {
    fn mono(&mut self, sample: f32);
    fn gen1(&mut self, sample: f32);
    fn gen2(&mut self, sample: f32);
    fn flush(&mut self);
}

pub struct AudioProcessingUnit {
    machine: Machine,

    pub s1: SquareWaveSoundGenerator,
    pub s2: SquareWaveSoundGenerator,
    pub ch3: WaveSoundGenerator,
    pub ch4: NoiseSoundGenerator,
    pub nr50: u8,
    pub nr51: u8,

    // Bit 7 of NR52. Controls power to the audio hardware
    pub powered_on: bool,

    // PREVIOUS METHOD:
    // Producer for the output ring buffer.
    // Every cycle one sample is appended to this buffer.
    //pub buf: Option<Producer<f32>>,

    // NEW METHOD using blip_buf:
    pub buf_left: BlipBuf,
    pub buf_right: BlipBuf,
    pub buf_clock: u32,
    pub buf_left_amp: i16,
    pub buf_right_amp: i16,

    pub recorder: Option<Box<dyn AudioRecorder>>,

    // Current frame sequencer step. Updated at 512 Hz,
    // or every 8192'th cycle.
    pub frame_seq_step: u8,
}

impl AudioProcessingUnit {
    pub fn new(machine: Machine, buf_size: u32) -> Self {
        AudioProcessingUnit {
            machine,
            s1: SquareWaveSoundGenerator::new(true, machine),
            s2: SquareWaveSoundGenerator::new(false, machine),
            ch3: WaveSoundGenerator::new(machine),
            ch4: NoiseSoundGenerator::new(machine),
            nr50: 0,
            nr51: 0,
            buf_left: BlipBuf::new(buf_size),
            buf_right: BlipBuf::new(buf_size),
            buf_clock: 0,
            buf_left_amp: 0,
            buf_right_amp: 0,
            recorder: None,
            powered_on: false,
            frame_seq_step: 0,
        }
    }

    // Perform a complete reset. Used when resetting the whole machine.
    // Note that the APU can't easily be recreated, as it has a ringbuf
    // producer that can't be moved to a new instance of it, so instead
    // we must reset all values.
    pub fn reset(&mut self) {
        self.s1 = SquareWaveSoundGenerator::new(true, self.machine);
        self.s2 = SquareWaveSoundGenerator::new(false, self.machine);
        self.ch3 = WaveSoundGenerator::new(self.machine);
        self.ch4 = NoiseSoundGenerator::new(self.machine);
        self.nr50 = 0;
        self.nr51 = 0;
        self.powered_on = false;
    }

    pub fn update_4t(&mut self, div_counter: u16) {
        // NR52 bit 7 is used to disable the sound system completely

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

        assert!(div_counter % 2 == 0);

        if div_counter % 8192 == 0 {
            self.frame_seq_step = (self.frame_seq_step + 1) & 7;
            hz64 = self.frame_seq_step == 7;
            hz128 = self.frame_seq_step == 2 || self.frame_seq_step == 6;
            hz256 = self.frame_seq_step & 1 == 0;
        }

        let ch1_output = self.s1.update_4t(hz64, hz128, hz256);
        let ch2_output = self.s2.update_4t(hz64, hz128, hz256);
        let ch3_output = self.ch3.update_4t(hz256);
        let ch4_output = self.ch4.update_4t(hz64, hz256);

        // Mixer
        let mut left: i16 = 0;
        if self.nr51 & 128 != 0 {
            left += ch4_output >> 2;
        }
        if self.nr51 & 64 != 0 {
            left += ch3_output >> 2;
        }
        if self.nr51 & 32 != 0 {
            left += ch2_output >> 2;
        }
        if self.nr51 & 16 != 0 {
            left += ch1_output >> 2;
        }

        let mut right: i16 = 0;
        if self.nr51 & 8 != 0 {
            right += ch4_output >> 2;
        }
        if self.nr51 & 4 != 0 {
            right += ch3_output >> 2;
        }
        if self.nr51 & 2 != 0 {
            right += ch2_output >> 2;
        }
        if self.nr51 & 1 != 0 {
            right += ch1_output >> 2;
        }

        // FIXME: Recorder is disabled for now
        // if let Some(ref mut rec) = self.recorder {
        //     rec.gen1(ch1_output as f32);
        //     rec.gen2(ch2_output as f32);
        //     rec.mono(sample);
        // }

        let left_delta = (left as i32) - (self.buf_left_amp as i32);
        let right_delta = (right as i32) - (self.buf_right_amp as i32);
        assert!(abs(left_delta) <= 0xffff);
        assert!(abs(right_delta) <= 0xffff);
        self.buf_left_amp = left;
        self.buf_right_amp = right;
        if left_delta != 0 {
            self.buf_left.add_delta(self.buf_clock, left_delta as i32);
        }
        if right_delta != 0 {
            self.buf_right.add_delta(self.buf_clock, right_delta as i32);
        }

        // Add left and right output to Blip buffer and increment buffer clock
        // if self.buf_left.samples_avail() == 0 {
        //     if self.buf_right.samples_avail() == 0 {
        //         eprintln!("Audio buffer is full");
        //     } else {
        //         // eprintln!("Left audio buffer is full");
        //         self.buf_right.add_delta(self.buf_clock, right as i32);
        //     }
        // } else {
        //     self.buf_left.add_delta(self.buf_clock, left as i32);
        //     if self.buf_right.samples_avail() == 0 {
        //         // eprintln!("Right audio buffer is full");
        //     } else {
        //         self.buf_right.add_delta(self.buf_clock, right as i32);
        //     }
        // }

        self.buf_clock = self.buf_clock.wrapping_add(1);
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
            0xFF10..=0xFF14 => self.s1.read_reg(address),
            0xFF15..=0xFF19 => self.s2.read_reg(address),
            0xFF1A..=0xFF1E => self.ch3.read_reg(address),
            0xFF1F..=0xFF23 => self.ch4.read_reg(address),
            NR50_REG => self.nr50,
            NR51_REG => self.nr51,
            NR52_REG => self.read_nr52() | 0b0111_0000,
            0xFF27..=0xFF2F => 0xFF,
            0xFF30..=0xFF3F => self.ch3.read_wave_reg(address as usize),
            _ => 0,
        }
    }

    fn power_on(&mut self) {
        if !self.powered_on {
            self.powered_on = true;

            // When powered on, the frame sequencer is reset so that
            // the next step will be 0.
            self.frame_seq_step = 7;
        }
    }

    fn power_off(&mut self) {
        if self.powered_on {
            self.powered_on = false;
            self.s1.power_off();
            self.s2.power_off();
            self.ch3.power_off_reset();
            self.ch4.power_off();
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
            0xFF30..=0xFF3F => self.ch3.write_wave_reg(address as usize, value),
            _ => {}
        }

        match address {
            0xFF10..=0xFF14 => {
                self.s1
                    .write_reg(address, value, self.frame_seq_step, self.powered_on)
            }
            0xFF15..=0xFF19 => {
                self.s2
                    .write_reg(address, value, self.frame_seq_step, self.powered_on)
            }
            0xFF1A..=0xFF1E => {
                self.ch3
                    .write_reg(address, value, self.frame_seq_step, self.powered_on)
            }
            0xFF1F => {}
            0xFF20..=0xFF23 => {
                self.ch4
                    .write_reg(address, value, self.frame_seq_step, self.powered_on)
            }
            NR50_REG => {
                if self.powered_on {
                    self.nr50 = value
                }
            }
            NR51_REG => {
                if self.powered_on {
                    self.nr51 = value
                }
            }
            0xFF27..=0xFF2F => {}
            _ => {}
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
