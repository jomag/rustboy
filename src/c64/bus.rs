// The bus implemented in this file is a hardcoded exact copy of
// the jomag/my6502 computer. It would probably be quite easy to
// make it more flexible.

use std::{
    fs::File,
    io::{self, Read},
};

use crate::MemoryMapped;

pub type ReadWrite = bool;
pub const RD: ReadWrite = true;
pub const WR: ReadWrite = false;

const RAM_SIZE: usize = 0x10000;
const KERNAL_ROM_SIZE: usize = 0x2000;
const CHAR_ROM_SIZE: usize = 0x1000;
const BASIC_ROM_SIZE: usize = 0x2000;
const CARTRIDGE_ROM_SIZE: usize = 0x4000;

pub struct Bus {
    pub kernal_rom: Box<[u8]>,
    pub basic_rom: Box<[u8]>,
    pub char_rom: Box<[u8]>,
    pub ram: Box<[u8]>,
    pub bank_switch_mode: u8,
    pub cartridge_rom: Box<[u8]>,
}

impl MemoryMapped for Bus {
    fn read(&self, adr: usize) -> u8 {
        // FIXME: This is a simplification. Only ROM and RAM are supported.
        match adr {
            0x0000..=0x0FFF => self.ram[adr],
            0x1000..=0x7FFF | 0xC000..=0xCFFF => match self.bank_switch_mode {
                16..=23 => 0,
                _ => self.ram[adr],
            },
            0x8000..=0x9FFF => match self.bank_switch_mode {
                3 | 7 | 11 | 15..=23 => self.cartridge_rom[adr - 0x8000], // NOTE: should be cartridge LO
                _ => self.ram[adr],
            },
            0xA000..=0xBFFF => match self.bank_switch_mode {
                11 | 15 | 27 | 31 => self.basic_rom[adr - 0xA000],
                2 | 3 | 6 | 7 => self.cartridge_rom[adr - 0xA000], // NOTE: should be cartridge HI
                _ => self.ram[adr],
            },
            0xD000..=0xDFFF => match self.bank_switch_mode {
                2 | 3 | 9 | 10 | 11 | 25 | 26 | 27 => self.char_rom[adr - 0xD000],
                5 | 6 | 7 | 13..=23 | 29..=31 => self.read_io(adr),
                _ => self.ram[adr],
            },
            0xE000..=0xFFFF => match self.bank_switch_mode {
                2 | 3 | 6 | 7 | 10 | 11 | 14 | 15 | 26 | 27 | 30 | 31 => {
                    self.kernal_rom[adr - 0xE000]
                }
                16..=23 => self.cartridge_rom[adr - 0xE000], // NOTE: should be cartridge HI
                _ => self.ram[adr],
            },
            _ => unreachable!(),
        }
    }

    fn write(&mut self, adr: usize, value: u8) {
        match adr {
            0x0000..=0x0FFF => self.ram[adr] = value,
            0x1000..=0x7FFF | 0xC000..=0xCFFF => match self.bank_switch_mode {
                16..=23 => {}
                _ => self.ram[adr] = value,
            },
            0x8000..=0x9FFF => match self.bank_switch_mode {
                3 | 7 | 11 | 15..=23 => {}
                _ => self.ram[adr] = value,
            },
            0xA000..=0xBFFF => match self.bank_switch_mode {
                11 | 15 | 27 | 31 => {}
                2 | 3 | 6 | 7 => {}
                _ => self.ram[adr] = value,
            },
            0xD000..=0xDFFF => match self.bank_switch_mode {
                2 | 3 | 9 | 10 | 11 | 25 | 26 | 27 => {}
                5 | 6 | 7 | 13..=23 | 29..=31 => self.write_io(adr, value),
                _ => self.ram[adr] = value,
            },
            0xE000..=0xFFFF => match self.bank_switch_mode {
                2 | 3 | 6 | 7 | 10 | 11 | 14 | 15 | 26 | 27 | 30 | 31 => {}
                16..=23 => {}
                _ => self.ram[adr] = value,
            },
            _ => unreachable!(),
        };
    }

    fn reset(&mut self) {
        todo!()
    }
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            kernal_rom: vec![0; KERNAL_ROM_SIZE].into_boxed_slice(),
            char_rom: vec![0; CHAR_ROM_SIZE].into_boxed_slice(),
            ram: vec![0; RAM_SIZE].into_boxed_slice(),
            basic_rom: vec![0; BASIC_ROM_SIZE].into_boxed_slice(),
            cartridge_rom: vec![0; CARTRIDGE_ROM_SIZE].into_boxed_slice(),
            bank_switch_mode: 0x1F,
        }
    }

    fn read_io(&self, _adr: usize) -> u8 {
        0xFE
    }

    fn write_io(&self, _adr: usize, _value: u8) {
        todo!();
    }

    pub fn load_kernal_rom(&mut self, path: &str) -> Result<(), io::Error> {
        let mut file = File::open(path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        self.kernal_rom = content.into_boxed_slice();
        return Ok(());
    }

    pub fn load_char_rom(&mut self, path: &str) -> Result<(), io::Error> {
        let mut file = File::open(path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        self.char_rom = content.into_boxed_slice();
        return Ok(());
    }

    pub fn load_basic_rom(&mut self, path: &str) -> Result<(), io::Error> {
        let mut file = File::open(path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        self.basic_rom = content.into_boxed_slice();
        return Ok(());
    }

    pub fn load(&mut self, buf: Vec<u8>, offset: usize) {
        if offset + buf.len() >= 0x10000 {
            panic!(
                "Buffer with size {} and offset {} does not fit in memory",
                buf.len(),
                offset
            );
        }

        // Heavily unoptimized, but it should be plenty fast enough anyway ...
        let mut adr = offset;
        for b in buf {
            self.write(adr, b);
            adr += 1;
        }
    }
}
