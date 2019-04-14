use mmu::{NR10_REG, NR11_REG, NR12_REG, NR13_REG, NR14_REG, NR50_REG, NR51_REG, NR52_REG};
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
            0 => self.nr10 = value,
            1 => self.nr11 = value,
            2 => self.nr12 = value,
            3 => self.nr13 = value,
            4 => self.nr14 = value,
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
        // let freq = 4194304 / (4 * 2 * (2048 - freq_raw));
        let freq = 131072 / (2048 - freq_raw);
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
}

pub struct AudioProcessingUnit {
    pub s1: SquareWaveSoundGenerator,
    pub s2: SquareWaveSoundGenerator,
    pub nr50: u8,
    pub nr51: u8,
    pub nr52: u8,
}

impl AudioProcessingUnit {
    pub fn new() -> Self {
        AudioProcessingUnit {
            s1: SquareWaveSoundGenerator::new(),
            s2: SquareWaveSoundGenerator::new(),
            nr50: 0,
            nr51: 0,
            nr52: 0,
        }
    }

    pub fn generate(&mut self, samples: usize) -> Vec<i16> {
        if self.nr52 & 0x80 == 0 {
            return Vec::new();
        }

        let mut mix: Vec<i16> = Vec::with_capacity(samples);
        for _ in 0..samples {
            mix.push(0);
        }

        if self.nr52 & 0x01 == 0 {
            let wave = self.s1.generate(samples);
            for i in 0..wave.len() {
                mix[i] = wave[i] / 2;
            }
        }

        if self.nr52 & 0x02 == 0 {
            let wave = self.s2.generate(samples);
            for i in 0..wave.len() {
                mix[i] = mix[i] + wave[i] / 2;
            }
        }

        return mix;
    }

    pub fn read_reg(&self, address: u16) -> u8 {
        match address {
            0xFF10...0xFF14 => self.s1.read_reg(address),
            0xFF15...0xFF19 => self.s2.read_reg(address),
            NR50_REG => self.nr50,
            NR51_REG => self.nr51,
            NR52_REG => self.nr52,
            _ => 0,
        }
    }

    pub fn write_reg(&mut self, address: u16, value: u8) {
        // println!("Write audio register 0x{:04X}: 0x{:02X}", address, value);
        match address {
            0xFF10...0xFF14 => self.s1.write_reg(address - 0xFF10, value),
            0xFF15...0xFF19 => self.s2.write_reg(address - 0xFF15, value),
            NR50_REG => {
                println!("NRF50 = {:02X}", value);
                self.nr50 = value
            }
            NR51_REG => {
                println!("NRF51 = {:02X}", value);
                self.nr51 = value
            }
            NR52_REG => {
                println!("NRF52 = {:02X}", value);
                self.nr52 = value
            }
            _ => {}
        }
    }
}
