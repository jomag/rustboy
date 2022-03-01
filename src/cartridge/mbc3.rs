use crate::mmu::MemoryMapped;

use super::{
    cartridge::Cartridge,
    cartridge_header::{CartridgeHeader, RAM_BANK_SIZE, ROM_BANK_SIZE},
    cartridge_type::{Aux, CartridgeType},
};
use chrono::{Datelike, Timelike};

struct RTC {
    second: u8,
    minute: u8,
    hour: u8,
    day_counter: u16,
    halted: bool,
    prep_latch: bool,
}

// Real-time clock as used in MBC3 cartridges.
//
// The RTC implementation is not perfect:
// - Time can not really be halted. When halted, the time will not
//   change until unhalted, but then it will assume the *current*
//   timestamp, rather than continuing from where it was
// - Setting time is not implemented at all
// - Resetting the day counter is also not implemented at al
// - As the day counter only returns number of days since epoch % 512,
//   the carry bit will never be set
impl RTC {
    fn new() -> Self {
        RTC {
            second: 0,
            minute: 0,
            hour: 0,
            day_counter: 0,
            halted: false,
            prep_latch: false,
        }
    }

    fn latch(&mut self) {
        let now = chrono::Local::now();
        self.second = now.second() as u8;
        self.minute = now.minute() as u8;
        self.hour = now.hour() as u8;
        self.day_counter = (now.date().num_days_from_ce() & 0x1FF) as u16;
    }

    fn read_register(&self, reg: u8) -> u8 {
        match reg {
            0x08 => self.second,
            0x09 => self.minute,
            0x0A => self.hour,
            0x0B => (self.day_counter & 0xff) as u8,
            0x0C => {
                let mut v = ((self.day_counter >> 8) & 1) as u8;
                if self.halted {
                    v |= 0b0100_0000;
                }
                // Note: carry will never be set with the current
                // naive implementation
                v
            }
            _ => panic!("Invalid RTC register: 0x{:02x}", reg),
        }
    }

    fn write_latch(&mut self, value: u8) {
        match value {
            0 => self.prep_latch = true,
            1 => {
                if self.prep_latch {
                    self.prep_latch = false;
                    if !self.halted {
                        self.latch();
                    }
                }
            }
            _ => panic!("Invalid RTC latch value: 0x{:02x}", value),
        }
    }

    fn write_register(&mut self, reg: u8, value: u8) {
        match reg {
            0x08 => self.second = value,
            0x09 => self.minute = value,
            0x0A => self.hour = value,
            0x0B => self.day_counter = (self.day_counter & 0x100) | value as u16,
            0x0C => {
                if value & 1 == 0 {
                    self.day_counter = value as u16;
                } else {
                    self.day_counter = value as u16 | 0x100
                }
                self.halted = value & 0b0100_0000 != 0;
            }
            _ => panic!("Invalid RTC register: 0x{:02x}", reg),
        }
    }
}

pub struct MBC3 {
    // Memory buffers
    pub rom: Box<[u8]>,
    pub ram: Option<Box<[u8]>>,
    rtc: Option<RTC>,

    // Current ROM (0x4000+) and RAM offsets
    rom_offset: usize,
    ram_offset: usize,

    // MBC registers
    rom_bank: u8,
    aux_enabled: bool,
    register_selection: u8,

    // Meta
    pub cartridge_type: CartridgeType,
    header: CartridgeHeader,
}

impl MBC3 {
    pub fn new(cartridge_type: CartridgeType, data: &Vec<u8>) -> Self {
        let header = CartridgeHeader::from_header(data);

        let mut rom = vec![0; header.rom_size].into_boxed_slice();
        for (src, dst) in rom.iter_mut().zip(data.iter()) {
            *src = *dst
        }

        let ram = match header.ram_size {
            0 => None,
            sz => Some(vec![0; sz].into_boxed_slice()),
        };

        let rtc = match cartridge_type {
            CartridgeType::MBC3 { rtc, .. } => Some(RTC::new()),
            _ => None,
        };

        let mut cartridge = MBC3 {
            rom,
            ram,
            rtc,
            rom_offset: 0,
            ram_offset: 0,
            rom_bank: 1,
            register_selection: 0,
            aux_enabled: false,
            cartridge_type,
            header,
        };

        cartridge.reset();
        cartridge
    }

    fn read_ram(&self, offset: usize) -> u8 {
        match self.aux_enabled {
            true => match &self.ram {
                Some(ram) => ram[self.ram_offset + offset],
                None => 0xFF,
            },
            false => 0,
        }
    }

    fn write_ram(&mut self, offset: usize, value: u8) {
        match self.aux_enabled {
            true => match &mut self.ram {
                Some(ram) => ram[self.ram_offset + offset] = value,
                None => {}
            },
            false => {}
        }
    }

    fn update_offsets(&mut self) {
        let rom_mask = self.header.rom_bank_count - 1;
        self.rom_offset = (self.rom_bank as usize & rom_mask) * ROM_BANK_SIZE;

        let bank_count = self.header.ram_bank_count;
        let ram_mask = if bank_count > 0 { bank_count - 1 } else { 0 };
        self.ram_offset = (self.register_selection as usize & ram_mask) * RAM_BANK_SIZE;
    }
}

impl MemoryMapped for MBC3 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => self.rom[self.rom_offset + address as usize - 0x4000],
            0xA000..=0xBFFF => match self.aux_enabled {
                true => match self.register_selection {
                    0x00..=0x03 => self.read_ram(address as usize - 0xA000),
                    0x0B..=0x0C => match &self.rtc {
                        Some(rtc) => rtc.read_register(self.register_selection),
                        None => 0xFF,
                    },
                    _ => 0xFF,
                },
                false => 0xFF,
            },
            _ => 0xFF,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.aux_enabled = value == 0x0A,
            0x2000..=0x3FFF => {
                let masked = value & 0b0111_1111;
                self.rom_bank = if masked == 0 { 1 } else { masked };
                self.update_offsets();
            }
            0x4000..=0x5FFF => {
                self.register_selection = value;
                self.update_offsets();
            }
            0x6000..=0x7FFF => {
                if self.aux_enabled {
                    if let Some(ref mut rtc) = self.rtc {
                        rtc.write_latch(value);
                    }
                }
            }
            0xA000..=0xBFFF => {
                if self.aux_enabled {
                    match self.register_selection {
                        0x00..=0x03 => self.write_ram(address as usize - 0xA000, value),
                        0x08..=0x0C => {
                            if let Some(ref mut rtc) = self.rtc {
                                rtc.write_register(self.register_selection, value);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        if let Some(ram) = &mut self.ram {
            ram.fill(0);
        }

        self.rom_bank = 1;
        self.register_selection = 0;
        self.aux_enabled = false;
        self.update_offsets();
    }
}

impl Cartridge for MBC3 {
    fn cartridge_type(&self) -> CartridgeType {
        return self.cartridge_type;
    }

    fn header(&self) -> &CartridgeHeader {
        &self.header
    }

    fn read_abs(&self, address: usize) -> u8 {
        return self.rom[address];
    }
}
