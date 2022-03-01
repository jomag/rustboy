use super::cartridge_type::Aux;
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

    fn read(&self, adr: usize, reg: u8) -> u8 {
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

    fn write(&mut self, adr: u16, reg: u8, value: u8) {
        match adr {
            0x6000..=0x7FFF => match value {
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
            },
            0xA000..=0xBFFF => match reg {
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
            },
            _ => panic!("Invalid RTC address"),
        }
    }
}

pub struct MBC3 {
    // Memory buffers
    pub rom: Box<[u8]>,
    pub ram: Option<Box<[u8]>>,

    // Current ROM and RAM offsets
    rom_offset_0x4000_0x7fff: usize,

    aux_selection: Aux,
    rtc: Option<RTC>,
    pub rtc_register: u8,
    ram_enabled: bool,
}

impl MBC3 {
    fn read_rtc(&self, offset: usize) -> u8 {
        match &self.rtc {
            Some(rtc) => rtc.read(offset, self.rtc_register),
            None => 0,
        }
    }

    pub fn reset(&mut self) {
        if let Some(ram) = &mut self.ram {
            ram.fill(0);
        }

        self.rtc_register = 0;
        self.ram_enabled = false;
    }

    fn read_ram(&self, offset: usize) -> u8 {
        0
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => self.rom[self.rom_offset_0x4000_0x7fff + address as usize - 0x4000],
            0xA000..=0xBFFF => match &self.aux_selection {
                RAM => self.read_ram(address as usize - 0xA000),
                RTC => self.read_rtc(address as usize - 0xA000),
            },
            _ => 0,
        }
    }
}
