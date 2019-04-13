use mmu::{NR10_REG, NR11_REG, NR12_REG, NR13_REG, NR14_REG};
use sdl2::audio::{AudioCallback, AudioQueue, AudioSpecDesired};
use sdl2::init;
use std::collections::VecDeque;
use std::thread::sleep;
use std::time::Duration;

pub struct SquareWaveSoundGenerator {
    duty: u32,
    period: u32,
    ctr: u32,
    volume: i16,
    sample_count: u32,
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
}

impl SquareWaveSoundGenerator {
    pub fn new() -> Self {
        SquareWaveSoundGenerator {
            duty: 2,
            period: 0,
            ctr: 0,
            volume: 16384,
            sample_count: 0,
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
        }
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            NR10_REG => self.nr10,
            NR11_REG => self.nr11,
            NR12_REG => self.nr12,
            NR13_REG => self.nr13,
            NR14_REG => self.nr14,
            _ => 0,
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8) {
        match address {
            NR10_REG => self.nr10 = value,
            NR11_REG => self.nr11 = value,
            NR12_REG => self.nr12 = value,
            NR13_REG => self.nr13 = value,
            NR14_REG => self.nr14 = value,
            _ => {}
        }
    }

    pub fn generate(&mut self, samples: usize) -> Vec<i16> {
        let mut buf = Vec::new();

        // FIXME: not true! writing to msb in nr14
        // should restart sound generator 1, even
        // if it is already active.
        if self.nr14 & 0x80 == 0 {
            return buf;
        }

        let duty_cycle = match self.nr10 >> 6 {
            0 => 12, // 12.5% ...
            1 => 25,
            2 => 50,
            3 => 75,
            _ => 12,
        };

        let freq_raw: u32 = ((self.nr13 as u16) | (((self.nr14 & 0x07) as u16) << 8)) as u32;
        let freq = 4194304 / (4 * 2 * (2048 - freq_raw));
        let freq_samples = 44100 / freq;

        for _ in 0..samples {
            if self.sample_count % freq_samples < (freq_samples / 2) {
                buf.push(3000);
            } else {
                buf.push(-3000);
            }
            self.sample_count = self.sample_count + 1
        }

        buf
    }

    pub fn xgenerate(&mut self, samples: usize) -> Vec<i16> {
        let mut volume = self.volume;
        let mut result = Vec::new();

        for _ in 0..samples {
            self.ctr += 1;

            if self.ctr % 10 == 0 {
                if volume > 5 {
                    volume = volume - 20;
                }
            }
            if self.ctr % 10000 == 0 {
                self.period = self.period + 1;
            }

            if self.period == 0 {
                result.push(0);
            } else {
                let pctr = self.ctr % self.period;

                if pctr * 8 < self.period * self.duty {
                    result.push(volume);
                } else {
                    result.push(-volume);
                }
            }
        }

        self.volume = volume;
        result
    }
}

pub struct AudioProcessingUnit {
    pub s1: SquareWaveSoundGenerator,
    pub nr24: u8,
    pub nr25: u8,
    pub nr26: u8,
}

impl AudioProcessingUnit {
    pub fn new() -> Self {
        AudioProcessingUnit {
            s1: SquareWaveSoundGenerator::new(),
            nr24: 0,
            nr25: 0,
            nr26: 0,
        }
    }

    pub fn generate(&mut self, samples: usize) -> Vec<i16> {
        if self.nr26 & 0x80 == 0 {
            return Vec::new();
        }

        if self.nr26 & 0x01 == 0 {
            return self.s1.generate(samples);
        }

        return Vec::new();
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            0xFF10...0xFF14 => self.s1.read_reg(address),
            0xFF24 => self.nr24,
            0xFF25 => self.nr25,
            0xFF26 => self.nr26,
            _ => 0,
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8) {
        // println!("Write audio register 0x{:04X}: 0x{:02X}", address, value);
        match address {
            0xFF10...0xFF14 => self.s1.write_reg(address, value),
            0xFF24 => self.nr24 = value,
            0xFF25 => self.nr25 = value,
            0xFF26 => self.nr26 = value,
            _ => {}
        }
    }
}

// pub fn sound_test() {
//     let sdl_context = init().unwrap();
//     let audio_subsystem = sdl_context.audio().unwrap();

//     let desired_spec = AudioSpecDesired {
//         freq: Some(44_100),
//         channels: Some(1),
//         samples: None,
//     };

//     let queue = audio_subsystem
//         .open_queue::<i16, _>(None, &desired_spec)
//         .unwrap();
//     queue.resume();
//     let mut apu = AudioProcessingUnit::new(queue);

//     let c4: f32 = 261.61;
//     let d4: f32 = 293.66;
//     let e4: f32 = 329.63;
//     let f4: f32 = 349.23;
//     let g4: f32 = 392.00;
//     let a4: f32 = 440.00;
//     let b4: f32 = 493.88;

//     let song = [
//         (e4, 150, 50),
//         (c4, 150, 50),
//         (d4, 150, 50),
//         (g4, 100, 0),
//         (e4, 200, 0),
//         (e4, 100, 0),
//         (c4, 150, 50),
//         (d4, 150, 50),
//         (g4, 150, 50),
//     ];

//     loop {
//         for note in song.iter() {
//             let freq = note.0;
//             let play = note.1;
//             let pause = note.2;

//             println!("Play {}", freq);
//             apu.s1.duty = apu.s1.duty + 1;
//             if apu.s1.duty == 8 {
//                 apu.s1.duty = 1
//             }
//             apu.s1.period = ((1.0 / freq) * 44100.0) as u32;
//             apu.s1.volume = 16384;
//             println!(" period: {}", apu.s1.period);
//             apu.run(play);

//             println!("...");
//             apu.s1.period = 0;
//             apu.run(pause);
//         }
//     }
// }
