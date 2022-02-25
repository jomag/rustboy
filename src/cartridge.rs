use std::fs::File;
use std::io::Read;

use crate::{conv, utils::VecExt};
use chrono::{Datelike, Timelike};

struct RTC {
    second: u8,
    minute: u8,
    hour: u8,
    day_counter: u16,
    halted: bool,
    prep_latch: bool,
}

pub fn is_mbc1_multicart(rom: &Box<[u8]>) -> bool {
    // There's nothing in the header that tells if the cartridge is
    // an multicart. All known multicarts are 8 Mbit. Bit 4 in the
    // first bank register is not connected, which reduces the number
    // of bank bits to 6. Check all banks if they contain the Nintendo
    // logo. If two or more banks do so, it's likely a multicart.
    // Given the above, the possible logo offsets are: 0x00104,
    // 0x40104, 0x80104 and 0xC0104
    const LOGO: [u8; 48] = [
        0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00,
        0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD,
        0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB,
        0xB9, 0x33, 0x3E,
    ];

    let validate_logo = |offset: usize| {
        for i in 0..48 {
            if rom[offset + i] != LOGO[i] {
                return false;
            }
        }
        return true;
    };

    let mut count = 0;
    for offset in [0x00104, 0x40104, 0x80104, 0xC0104] {
        if validate_logo(offset) {
            count += 1;
        }
    }

    return count > 1;
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

pub enum CartridgeType {
    NoCartridge,
    NoMBC { ram: bool, bat: bool },
    MBC1 { ram: bool, bat: bool },
    MBC2 { ram: bool, bat: bool },
    MBC3 { ram: bool, bat: bool, rtc: bool },
    MBC5 { ram: bool, bat: bool, rumble: bool },
    MBC6,
    MBC7,
    MMM01 { ram: bool, bat: bool },
    PocketCamera,
    BandaiTAMA5,
    HuC1,
    HuC3,
}

pub enum Aux {
    RAM,
    RTC,
}

fn aux_string(name: &str, ram: bool, bat: bool, rtc: bool, rumble: bool) -> String {
    let mut extras: Vec<&str> = vec![];
    extras.push_if(ram, "RAM");
    extras.push_if(bat, "battery");
    extras.push_if(rtc, "RTC");
    extras.push_if(rumble, "rumble");

    if let Some((last, rest)) = extras.split_last() {
        if rest.len() == 0 {
            format!("{} with {}", name, last)
        } else {
            format!("{} with {} and {}", name, rest.join(", "), last)
        }
    } else {
        name.to_string()
    }
}

impl CartridgeType {
    fn from_code(code: u8) -> Option<CartridgeType> {
        use self::CartridgeType::*;
        match code {
            0x00 | 0x08 | 0x09 => Some(NoMBC {
                ram: code != 0x00,
                bat: code == 0x09,
            }),
            0x01..=0x03 => Some(MBC1 {
                ram: code > 0x01,
                bat: code == 0x03,
            }),
            0x05 | 0x06 => Some(MBC2 {
                ram: true,
                bat: code == 0x06,
            }),
            0x0b..=0x0d => Some(MMM01 {
                ram: code > 0x0b,
                bat: code == 0x0d,
            }),
            0x0f..=0x13 => Some(MBC3 {
                ram: code == 0x10 || code == 0x12 || code == 0x13,
                bat: code == 0x10 || code == 0x13,
                rtc: code == 0x0f || code == 0x10,
            }),
            0x19..=0x1e => Some(MBC5 {
                ram: code != 0x19 && code != 0x1c,
                bat: code == 0x1b || code == 0x1e,
                rumble: code >= 0x1c,
            }),
            0x20 => Some(MBC6),
            0x22 => Some(MBC7),
            0xFC => Some(PocketCamera),
            0xFD => Some(BandaiTAMA5),
            0xFE => Some(HuC3),
            0xFF => Some(HuC1),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        use self::CartridgeType::*;
        match self {
            NoCartridge => "No cartridge".to_string(),
            NoMBC {
                ram: false,
                bat: false,
            } => "ROM only".to_string(),
            NoMBC {
                ram: true,
                bat: false,
            } => "ROM and RAM".to_string(),
            NoMBC {
                ram: false,
                bat: true,
            } => "ROM and RAM".to_string(),
            NoMBC {
                ram: true,
                bat: true,
            } => "ROM, RAM and battery".to_string(),
            MBC1 { ram, bat } => aux_string("MBC1", *ram, *bat, false, false),
            MBC2 { ram, bat } => aux_string("MBC2", *ram, *bat, false, false),
            MBC3 { ram, bat, rtc } => aux_string("MBC3", *ram, *bat, *rtc, false),
            MBC5 { ram, bat, rumble } => aux_string("MBC5", *ram, *bat, false, *rumble),
            MBC6 => "MBC6".to_string(),
            MBC7 => "MBC7 with RAM, sensor, rumble and battery".to_string(),
            MMM01 { ram, bat } => aux_string("MMM01", *ram, *bat, false, false),
            PocketCamera => "Pocket Camera".to_string(),
            BandaiTAMA5 => "Bandai TAMA5".to_string(),
            HuC3 => "HuC3".to_string(),
            HuC1 => "HuC1 with RAM and battery".to_string(),
        }
    }

    pub fn max_rom_size(&self) -> usize {
        use self::CartridgeType::*;
        match self {
            NoMBC { .. } => 32 * 1024,      // 32 KiB
            MBC1 { .. } => 2 * 1024 * 1024, // 2 MiB
            MBC2 { .. } => 256 * 1024,      // 256 KiB
            MBC3 { .. } => 2 * 1024 * 1024, // 2 MiB
            MBC5 { .. } => 8 * 1024 * 1024, // 8 MiB
            _ => panic!("Not implemented for {}", self.to_string()),
        }
    }

    pub fn max_ram_size(&self) -> usize {
        use self::CartridgeType::*;
        match self {
            NoMBC { ram: false, .. } => 0,
            NoMBC { ram: true, .. } => 0x2000,
            MBC1 { ram: false, .. } => 0,
            MBC1 { ram: true, .. } => 32 * 1024,
            MBC2 { ram: false, .. } => 0,
            MBC2 { ram: true, .. } => 512,
            MBC3 { ram: false, .. } => 0,
            MBC3 { ram: true, .. } => 32 * 1024,
            MBC5 { ram: false, .. } => 0,
            MBC5 { ram: true, .. } => 128 * 1024,
            _ => panic!("Not implemented for {}", self.to_string()),
        }
    }

    fn has_rtc(&self) -> bool {
        use self::CartridgeType::*;
        match self {
            MBC3 { rtc, .. } => *rtc,
            _ => false,
        }
    }
}

pub struct MBC1 {
    pub ram_enabled: bool,
    pub bank1: u8,
    pub bank2: u8,
    pub mode: u8,
    pub multicart: bool,
}

impl MBC1 {
    fn new(multicart: bool) -> Self {
        MBC1 {
            ram_enabled: false,
            bank1: 1,
            bank2: 0,
            mode: 0,
            multicart,
        }
    }

    fn reset(&mut self) {
        self.ram_enabled = false;
        self.bank1 = 0;
        self.bank2 = 0;
        self.mode = 0;
    }
}

pub struct Cartridge {
    pub cartridge_type: CartridgeType,
    aux_selection: Aux,

    pub rom: Box<[u8]>,
    pub ram: Option<Box<[u8]>>,
    rtc: Option<RTC>,

    pub rtc_register: u8,

    pub mbc1: MBC1,

    pub rom_offset_0x0000_0x3fff: usize,
    pub rom_offset_0x4000_0x7fff: usize,
    ram_offset: usize,
}

impl Cartridge {
    pub fn none() -> Self {
        Cartridge {
            cartridge_type: CartridgeType::NoCartridge,
            aux_selection: Aux::RAM,
            rtc_register: 0,
            rom: vec![0; 0].into_boxed_slice(),
            ram: None,
            rtc: None,
            mbc1: MBC1::new(false),

            ram_offset: 0,
            rom_offset_0x0000_0x3fff: 0,
            rom_offset_0x4000_0x7fff: 0,
        }
    }

    pub fn new(data: Vec<u8>) -> Self {
        match CartridgeType::from_code(data[0x147]) {
            None => panic!("Unsupported cartridge type"),
            Some(cartridge_type) => {
                let max_rom_size = cartridge_type.max_rom_size();
                if data.len() > max_rom_size {
                    panic!("ROM size too big to fit in cartridge");
                }

                let mut rom = vec![0; max_rom_size].into_boxed_slice();
                for (src, dst) in rom.iter_mut().zip(data.iter()) {
                    *src = *dst
                }

                let max_ram_size = cartridge_type.max_ram_size();
                let ram = match max_ram_size {
                    0 => None,
                    _ => Some(vec![0; max_ram_size].into_boxed_slice()),
                };

                let rtc = match cartridge_type.has_rtc() {
                    true => Some(RTC::new()),
                    false => None,
                };

                let multicart = match cartridge_type {
                    CartridgeType::MBC1 { .. } => is_mbc1_multicart(&rom),
                    _ => false,
                };

                let mut cartridge = Cartridge {
                    cartridge_type,
                    aux_selection: Aux::RAM,
                    rtc,
                    rom,
                    ram,
                    rtc_register: 0,
                    mbc1: MBC1::new(multicart),
                    ram_offset: 0,
                    rom_offset_0x0000_0x3fff: 0,
                    rom_offset_0x4000_0x7fff: 0,
                };

                cartridge.update_offsets();
                cartridge
            }
        }
    }

    pub fn rom_bank_count(&self) -> usize {
        match self.rom[0x148] {
            0..=8 => 2 << self.rom[0x148],
            n => {
                println!("Unknown ROM size code in cartridge header: 0x{:02X}", n);
                0
            }
        }
    }

    pub fn rom_size(&self) -> usize {
        self.rom_bank_count() * conv::kib(16)
    }

    pub fn ram_bank_count(&self) -> usize {
        match self.rom[0x0149] {
            0 => 0,
            2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            n => {
                println!("Unknown RAM size code in cartridge header: 0x{:02X}", n);
                0
            }
        }
    }

    pub fn ram_size(&self) -> usize {
        self.ram_bank_count() * conv::kib(8)
    }

    pub fn reset(&mut self) {
        if let Some(ram) = &mut self.ram {
            ram.fill(0);
        }

        self.rtc_register = 0;
        self.mbc1.reset();
        self.update_offsets()
    }

    pub fn selected_ram_bank(&self) -> usize {
        let bank_count = self.ram_bank_count();

        let bank_mask = if bank_count > 0 {
            (bank_count - 1) as u8
        } else {
            0
        };

        if self.mbc1.mode == 0 {
            return 0;
        }

        if self.rom_size() >= conv::MIB / 8 {
            return 0;
        } else {
            return (self.mbc1.bank2 & 0b11 & bank_mask) as usize;
        };
    }

    fn update_offsets(&mut self) {
        let bank_mask = self.rom_bank_count() - 1;

        self.rom_offset_0x0000_0x3fff = if self.mbc1.mode == 0 {
            0
        } else {
            if self.mbc1.multicart {
                (((self.mbc1.bank2 as usize) << 4) & bank_mask) << 14
            } else {
                (((self.mbc1.bank2 as usize) << 5) & bank_mask) << 14
            }
        };

        self.rom_offset_0x4000_0x7fff = if self.mbc1.multicart {
            ((((self.mbc1.bank2 << 4) | (self.mbc1.bank1 & 0b1111)) as usize) & bank_mask) << 14
        } else {
            ((((self.mbc1.bank2 << 5) | self.mbc1.bank1) as usize) & bank_mask) << 14
        };

        self.ram_offset = self.selected_ram_bank() * conv::kib(8);
    }

    fn read_ram(&self, offset: usize) -> u8 {
        match &self.ram {
            Some(ram) => match self.mbc1.ram_enabled {
                true => ram[self.ram_offset + offset],
                false => 0xFF,
            },
            None => 0xFF,
        }
    }

    fn write_ram(&mut self, offset: usize, value: u8) {
        match &mut self.ram {
            Some(ram) => match self.mbc1.ram_enabled {
                true => ram[self.ram_offset + offset] = value,
                false => {}
            },
            None => {}
        }
    }

    fn read_rtc(&self, offset: usize) -> u8 {
        match &self.rtc {
            Some(rtc) => rtc.read(offset, self.rtc_register),
            None => 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        use Aux::*;
        use CartridgeType::*;

        let adr = address as usize;

        match self.cartridge_type {
            NoMBC { .. } => match adr {
                0x0000..=0x7FFF => self.rom[adr],
                0xA000..=0xBFFF => match &self.ram {
                    Some(ram) => ram[adr],
                    None => 0x00,
                },
                _ => 0,
            },

            MBC1 { .. } => match adr {
                0x0000..=0x3FFF => self.rom[self.rom_offset_0x0000_0x3fff + adr],
                0x4000..=0x7FFF => self.rom[self.rom_offset_0x4000_0x7fff + (adr & 0x3fff)],
                0xA000..=0xBFFF => self.read_ram(adr - 0xA000),
                _ => 0,
            },

            MBC3 { .. } => match adr {
                0x0000..=0x3FFF => self.rom[adr],
                0x4000..=0x7FFF => self.rom[self.rom_offset_0x4000_0x7fff + adr - 0x4000],
                0xA000..=0xBFFF => match self.aux_selection {
                    RAM => self.read_ram(adr - 0xA000),
                    RTC => self.read_rtc(adr - 0xA000),
                },
                _ => 0,
            },

            _ => panic!("Unsupported cartridge type in read()"),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        use CartridgeType::*;
        let adr = address as usize;

        match self.cartridge_type {
            NoMBC { .. } => match adr {
                0xA000..=0xBFFF => {
                    if let Some(ref mut ram) = self.ram {
                        ram[adr - 0xA000] = value;
                    }
                }
                _ => {}
            },

            MBC1 { .. } => match adr {
                0x0000..=0x1FFF => {
                    self.mbc1.ram_enabled = value & 0x0F == 0x0A;
                    self.update_offsets();
                }
                0x2000..=0x3FFF => {
                    let masked = value & 0b11111;
                    self.mbc1.bank1 = if masked == 0 { 1 } else { masked };
                    self.update_offsets();
                }
                0x4000..=0x5FFF => {
                    self.mbc1.bank2 = value & 0b11;
                    self.update_offsets();
                }
                0x6000..=0x7FFF => {
                    self.mbc1.mode = value & 1;
                    self.update_offsets();
                }
                0xA000..=0xBFFF => {
                    self.write_ram(adr - 0xA000, value);
                }

                _ => {}
            },

            _ => panic!("Unsupported cartridge type in write()"),
        }
    }
}

pub fn load_cartridge(filename: String) -> Box<Cartridge> {
    let mut file = File::open(filename).unwrap();
    let mut rom: Vec<u8> = Vec::new();

    // Returns amount of bytes read and append the rebsult to the buffer
    file.read_to_end(&mut rom).unwrap();

    let code = rom[0x147];
    match CartridgeType::from_code(code) {
        None => println!("Unsupported cartridge type: 0x{:02x}", code),
        Some(t) => println!("Cartridge type 0x{:02x}: {}", code, t.to_string()),
    }

    return Box::new(Cartridge::new(rom));
}
