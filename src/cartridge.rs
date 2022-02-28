use std::fs::File;
use std::io::Read;
use std::str;

use crate::{conv, utils::VecExt};
use chrono::{Datelike, Timelike};

const ROM_BANK_SIZE: usize = 16384;
const RAM_BANK_SIZE: usize = 8192;

pub trait MemoryMapped {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);

    // Perform reset as after power cycle
    fn reset(&mut self);
}

pub struct CartridgeHeader {
    pub licensee_code: [u8; 2],
    pub old_licensee_code: u8,
    pub checksum: u8,
    pub global_checksum: u16,
    pub sgb_features: bool,
    pub cartridge_type: u8,
    pub rom_bank_count: usize,
    pub rom_size: usize,
    pub ram_bank_count: usize,
    pub ram_size: usize,
}

impl CartridgeHeader {
    fn from_header(header: &Vec<u8>) -> Self {
        let licensee_code: [u8; 2] = [header[0x144], header[0x145]];

        let rom_bank_count = match header[0x148] {
            0..=8 => 2 << header[0x148],
            _ => 0,
        };

        let ram_bank_count = match header[0x0149] {
            0 => 0,
            2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            _ => 0,
        };

        CartridgeHeader {
            licensee_code,
            old_licensee_code: header[0x14B],
            checksum: header[0x14D],
            global_checksum: ((header[0x14E] as u16) << 8) | header[0x14F] as u16,
            sgb_features: header[0x146] == 0x03,
            cartridge_type: header[0x147],
            rom_bank_count,
            ram_bank_count,
            rom_size: rom_bank_count * ROM_BANK_SIZE,
            ram_size: ram_bank_count * RAM_BANK_SIZE,
        }
    }

    pub fn licensee(&self) -> String {
        match str::from_utf8(&self.licensee_code) {
            Ok("00") => "None",
            Ok("01") => "Nintendo R&D1",
            Ok("08") => "Capcom",
            Ok("13") => "Electronic Arts",
            Ok("18") => "Hudson Soft",
            Ok("19") => "b-ai",
            Ok("20") => "kss",
            Ok("22") => "pow",
            Ok("24") => "PCM Complete",
            Ok("25") => "san-x",
            Ok("28") => "Kemco Japan",
            Ok("29") => "seta",
            Ok("30") => "Viacom",
            Ok("31") => "Nintendo",
            Ok("32") => "Bandai",
            Ok("33") => "Ocean/Acclaim",
            Ok("34") => "Konami",
            Ok("35") => "Hector",
            Ok("37") => "Taito",
            Ok("38") => "Hudson",
            Ok("39") => "Banpresto",
            Ok("41") => "Ubi Soft",
            Ok("42") => "Atlus",
            Ok("44") => "Malibu",
            Ok("46") => "angel",
            Ok("47") => "Bullet-Proof",
            Ok("49") => "irem",
            Ok("50") => "Absolute",
            Ok("51") => "Acclaim",
            Ok("52") => "Activision",
            Ok("53") => "American sammy",
            Ok("54") => "Konami",
            Ok("55") => "Hi tech entertainment",
            Ok("56") => "LJN",
            Ok("57") => "Matchbox",
            Ok("58") => "Mattel",
            Ok("59") => "Milton Bradley",
            Ok("60") => "Titus",
            Ok("61") => "Virgin",
            Ok("64") => "LucasArts",
            Ok("67") => "Ocean",
            Ok("69") => "Electronic Arts",
            Ok("70") => "Infogrames",
            Ok("71") => "Interplay",
            Ok("72") => "Broderbund",
            Ok("73") => "sculptured",
            Ok("75") => "sci",
            Ok("78") => "THQ",
            Ok("79") => "Accolade",
            Ok("80") => "misawa",
            Ok("83") => "lozc",
            Ok("86") => "Tokuma Shoten Intermedia",
            Ok("87") => "Tsukuda Original",
            Ok("91") => "Chunsoft",
            Ok("92") => "Video system",
            Ok("93") => "Ocean/Acclaim",
            Ok("95") => "Varie",
            Ok("96") => "Yonezawa/s’pal",
            Ok("97") => "Kaneko",
            Ok("99") => "Pack in soft",
            Ok("A4") => "Konami (Yu-Gi-Oh!)",
            Ok(_) => "Unknown",
            Err(_) => "Unknown",
        }
        .to_string()
    }
}

struct RTC {
    second: u8,
    minute: u8,
    hour: u8,
    day_counter: u16,
    halted: bool,
    prep_latch: bool,
}

pub fn is_mbc1_multicart(rom: &Vec<u8>) -> bool {
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
            if rom.len() < offset + i || rom[offset + i] != LOGO[i] {
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

#[derive(Copy, Clone)]
pub enum CartridgeType {
    NoCartridge,
    NoMBC {
        ram: bool,
        bat: bool,
    },
    MBC1 {
        ram: bool,
        bat: bool,
        multicart: bool,
    },
    MBC2 {
        bat: bool,
    },
    MBC3 {
        ram: bool,
        bat: bool,
        rtc: bool,
    },
    MBC5 {
        ram: bool,
        bat: bool,
        rumble: bool,
    },
    MBC6,
    MBC7,
    MMM01 {
        ram: bool,
        bat: bool,
    },
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
    fn from_rom(rom: &Vec<u8>) -> Option<CartridgeType> {
        use self::CartridgeType::*;
        let code = rom[0x147];
        match code {
            0x00 | 0x08 | 0x09 => Some(NoMBC {
                ram: code != 0x00,
                bat: code == 0x09,
            }),
            0x01..=0x03 => Some(MBC1 {
                ram: code > 0x01,
                bat: code == 0x03,
                multicart: is_mbc1_multicart(rom),
            }),
            0x05 | 0x06 => Some(MBC2 { bat: code == 0x06 }),
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
            MBC1 {
                ram,
                bat,
                multicart: true,
            } => aux_string("MBC1M", *ram, *bat, false, false),
            MBC1 {
                ram,
                bat,
                multicart: false,
            } => aux_string("MBC1", *ram, *bat, false, false),
            MBC2 { bat } => aux_string("MBC2", true, *bat, false, false),
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
            MBC2 { .. } => 512,
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

pub struct NoCartridge {}

impl MemoryMapped for NoCartridge {
    fn read(&self, address: u16) -> u8 {
        0
    }

    fn write(&mut self, address: u16, value: u8) {}
    fn reset(&mut self) {}
}

impl Cartridge for NoCartridge {
    fn cartridge_type(&self) -> CartridgeType {
        CartridgeType::NoCartridge
    }

    fn read_abs(&self, address: usize) -> u8 {
        0
    }

    fn header(&self) -> &CartridgeHeader {
        panic!("Can't return header when there's no cartridge in place")
    }
}

pub struct NoMBC {
    // Memory buffers
    pub rom: Box<[u8]>,
    pub ram: Option<Box<[u8]>>,
    cartridge_type: CartridgeType,
    header: CartridgeHeader,
}

impl NoMBC {
    fn new(cartridge_type: CartridgeType, data: &Vec<u8>) -> Self {
        let max_rom_size = cartridge_type.max_rom_size();
        let mut rom = vec![0; max_rom_size].into_boxed_slice();
        for (src, dst) in rom.iter_mut().zip(data.iter()) {
            *src = *dst
        }

        let max_ram_size = cartridge_type.max_ram_size();
        let ram = match max_ram_size {
            0 => None,
            _ => Some(vec![0; max_ram_size].into_boxed_slice()),
        };

        NoMBC {
            rom,
            ram,
            cartridge_type,
            header: CartridgeHeader::from_header(data),
        }
    }
}

impl MemoryMapped for NoMBC {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.rom[address as usize],
            0xA000..=0xBFFF => match &self.ram {
                Some(ram) => ram[address as usize],
                None => 0xFF,
            },
            _ => 0xFF,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0xA000..=0xBFFF => {
                if let Some(ref mut ram) = self.ram {
                    ram[address as usize - 0xA000] = value;
                }
            }
            _ => {}
        }
    }

    fn reset(&mut self) {}
}

impl Cartridge for NoMBC {
    fn cartridge_type(&self) -> CartridgeType {
        self.cartridge_type
    }

    fn read_abs(&self, address: usize) -> u8 {
        self.rom[address]
    }

    fn header(&self) -> &CartridgeHeader {
        &self.header
    }
}

pub struct MBC1 {
    // Memory buffers
    pub rom: Box<[u8]>,
    pub ram: Option<Box<[u8]>>,

    // Current ROM and RAM offsets
    rom_offset_0x0000_0x3fff: usize,
    rom_offset_0x4000_0x7fff: usize,
    ram_offset: usize,

    // MBC registers
    pub ram_enabled: bool,
    pub bank1: u8,
    pub bank2: u8,
    pub mode: u8,

    // Meta
    pub cartridge_type: CartridgeType,
    header: CartridgeHeader,
}

impl Cartridge for MBC1 {
    fn read_abs(&self, address: usize) -> u8 {
        return self.rom[address];
    }

    fn cartridge_type(&self) -> CartridgeType {
        return self.cartridge_type;
    }

    fn header(&self) -> &CartridgeHeader {
        &self.header
    }
}

impl MBC1 {
    fn new(cartridge_type: CartridgeType, data: &Vec<u8>) -> Self {
        let mut rom = vec![0; data.len()].into_boxed_slice();
        for (src, dst) in rom.iter_mut().zip(data.iter()) {
            *src = *dst
        }

        let max_ram_size = cartridge_type.max_ram_size();
        let ram = match max_ram_size {
            0 => None,
            _ => Some(vec![0; max_ram_size].into_boxed_slice()),
        };

        let mut cartridge = MBC1 {
            rom,
            ram,
            rom_offset_0x0000_0x3fff: 0,
            rom_offset_0x4000_0x7fff: 0,
            ram_offset: 0,
            ram_enabled: false,
            bank1: 0,
            bank2: 0,
            mode: 0,
            cartridge_type,
            header: CartridgeHeader::from_header(data),
        };

        cartridge.reset();
        cartridge
    }

    fn is_multicart(&self) -> bool {
        matches!(
            self.cartridge_type,
            CartridgeType::MBC1 {
                multicart: true,
                ..
            }
        )
    }

    pub fn selected_ram_bank(&self) -> usize {
        let bank_count = self.header.ram_bank_count;
        let bank_mask = if bank_count > 0 {
            (bank_count - 1) as u8
        } else {
            0
        };

        if self.mode == 0 {
            return 0;
        }

        if self.header.rom_size >= conv::MIB / 8 {
            return 0;
        } else {
            return (self.bank2 & 0b11 & bank_mask) as usize;
        };
    }

    fn update_offsets(&mut self) {
        let bank_mask = self.header.rom_bank_count - 1;

        if self.is_multicart() {
            self.rom_offset_0x0000_0x3fff = (((self.bank2 as usize) << 4) & bank_mask) << 14;
            self.rom_offset_0x4000_0x7fff =
                ((((self.bank2 << 4) | (self.bank1 & 0b1111)) as usize) & bank_mask) << 14;
        } else {
            self.rom_offset_0x0000_0x3fff = (((self.bank2 as usize) << 5) & bank_mask) << 14;
            self.rom_offset_0x4000_0x7fff =
                ((((self.bank2 << 5) | self.bank1) as usize) & bank_mask) << 14;
        }

        if self.mode == 0 {
            self.rom_offset_0x0000_0x3fff = 0;
        }

        self.ram_offset = self.selected_ram_bank() * conv::kib(8);
    }

    fn read_ram(&self, offset: usize) -> u8 {
        match &self.ram {
            Some(ram) => match self.ram_enabled {
                true => ram[self.ram_offset + offset],
                false => 0xFF,
            },
            None => 0xFF,
        }
    }

    fn write_ram(&mut self, offset: usize, value: u8) {
        match &mut self.ram {
            Some(ram) => match self.ram_enabled {
                true => ram[self.ram_offset + offset] = value,
                false => {}
            },
            None => {}
        }
    }
}

impl MemoryMapped for MBC1 {
    fn read(&self, address: u16) -> u8 {
        let adr = address as usize;
        match adr {
            0x0000..=0x3FFF => self.rom[self.rom_offset_0x0000_0x3fff + adr],
            0x4000..=0x7FFF => self.rom[self.rom_offset_0x4000_0x7fff + adr - 0x4000],
            0xA000..=0xBFFF => self.read_ram(adr - 0xA000),
            _ => 0,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.ram_enabled = value & 0xF == 0xA;
                self.update_offsets();
            }
            0x2000..=0x3FFF => {
                let masked = value & 0b11111;
                self.bank1 = if masked == 0 { 1 } else { masked };
                self.update_offsets();
            }
            0x4000..=0x5FFF => {
                self.bank2 = value & 0b11;
                self.update_offsets();
            }
            0x6000..=0x7FFF => {
                self.mode = value & 1;
                self.update_offsets();
            }
            0xA000..=0xBFFF => {
                self.write_ram(address as usize - 0xA000, value);
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.ram_enabled = false;
        self.bank1 = 1;
        self.bank2 = 0;
        self.mode = 0;
        self.update_offsets();
    }
}

pub struct MBC2 {
    // Memory buffers
    pub rom: Box<[u8]>,
    pub ram: Box<[u8]>,

    // Current ROM and RAM offsets
    rom_offset_0x4000_0x7fff: usize,

    // MBC registers
    pub ram_enabled: bool,
    pub bank: u8,

    // Meta
    pub cartridge_type: CartridgeType,
    header: CartridgeHeader,
}

impl MBC2 {
    fn new(cartridge_type: CartridgeType, data: &Vec<u8>) -> Self {
        let max_rom_size = cartridge_type.max_rom_size();
        let mut rom = vec![0; max_rom_size].into_boxed_slice();
        for (src, dst) in rom.iter_mut().zip(data.iter()) {
            *src = *dst
        }

        let max_ram_size = cartridge_type.max_ram_size();
        let ram = vec![0; max_ram_size].into_boxed_slice();

        let mut cartridge = MBC2 {
            rom,
            ram,
            ram_enabled: false,
            bank: 1,
            rom_offset_0x4000_0x7fff: 0,
            cartridge_type,
            header: CartridgeHeader::from_header(data),
        };

        cartridge.reset();
        cartridge
    }

    fn update_offsets(&mut self) {
        let bank_mask = self.header.rom_bank_count - 1;
        self.rom_offset_0x4000_0x7fff = ((self.bank as usize) & bank_mask) << 14;
    }
}

impl Cartridge for MBC2 {
    fn cartridge_type(&self) -> CartridgeType {
        self.cartridge_type
    }

    fn read_abs(&self, address: usize) -> u8 {
        self.rom[address]
    }

    fn header(&self) -> &CartridgeHeader {
        &self.header
    }
}

impl MemoryMapped for MBC2 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => self.rom[self.rom_offset_0x4000_0x7fff + address as usize - 0x4000],
            0xA000..=0xBFFF => match self.ram_enabled {
                true => self.ram[(address as usize - 0xA000) & 0x1ff] | 0xF0,
                false => 0xFF,
            },
            _ => 0,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3FFF => {
                if address & 0x100 == 0 {
                    self.ram_enabled = value & 0xF == 0xA;
                    self.update_offsets();
                } else {
                    self.bank = if value & 0xF == 0 { 1 } else { value & 0xF };
                    self.update_offsets();
                }
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    self.ram[(address as usize - 0xA000) & 0x1ff] = value;
                }
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.ram_enabled = false;
        self.bank = 1;
        self.update_offsets();
    }
}

pub struct MBC5 {
    // Memory buffers
    pub rom: Box<[u8]>,
    pub ram: Option<Box<[u8]>>,

    // Current ROM and RAM offsets
    rom_offset_0x4000_0x7fff: usize,
    ram_offset: usize,

    // MBC registers
    pub ram_enabled: bool,
    pub ram_bank: usize,
    pub rom_bank: usize,

    // Meta
    pub cartridge_type: CartridgeType,
    header: CartridgeHeader,
}

impl MBC5 {
    fn new(cartridge_type: CartridgeType, data: &Vec<u8>) -> Self {
        let header = CartridgeHeader::from_header(data);

        let mut rom = vec![0; header.rom_size].into_boxed_slice();
        for (src, dst) in rom.iter_mut().zip(data.iter()) {
            *src = *dst
        }

        let ram = match header.ram_size {
            0 => None,
            sz => Some(vec![0; sz].into_boxed_slice()),
        };

        let mut cartridge = MBC5 {
            rom,
            ram,
            ram_bank: 0,
            rom_bank: 1,
            ram_offset: 0,
            rom_offset_0x4000_0x7fff: 0,
            ram_enabled: false,
            cartridge_type,
            header,
        };

        cartridge.reset();
        cartridge
    }

    fn read_ram(&self, offset: usize) -> u8 {
        match self.ram_enabled {
            true => match &self.ram {
                Some(ram) => ram[self.ram_offset + offset as usize],
                None => 0xFF,
            },
            false => 0xFF,
        }
    }

    fn write_ram(&mut self, offset: usize, value: u8) {
        match self.ram_enabled {
            true => match &mut self.ram {
                Some(ram) => ram[self.ram_offset + offset as usize] = value,
                None => {}
            },
            false => {}
        }
    }

    fn update_offsets(&mut self) {
        let rom_mask = self.header.rom_bank_count - 1;

        let bank_count = self.header.ram_bank_count;
        let ram_mask = if bank_count > 0 { bank_count - 1 } else { 0 };

        self.rom_offset_0x4000_0x7fff = (self.rom_bank & rom_mask) * ROM_BANK_SIZE;
        self.ram_offset = (self.ram_bank & ram_mask) * RAM_BANK_SIZE;
    }
}

impl Cartridge for MBC5 {
    fn cartridge_type(&self) -> CartridgeType {
        self.cartridge_type
    }

    fn header(&self) -> &CartridgeHeader {
        &self.header
    }

    fn read_abs(&self, address: usize) -> u8 {
        self.rom[address]
    }
}

impl MemoryMapped for MBC5 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => self.rom[self.rom_offset_0x4000_0x7fff + address as usize - 0x4000],
            0xA000..=0xBFFF => self.read_ram(address as usize - 0xA000),
            _ => 0xFF,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = value == 0x0A,
            0x2000..=0x2FFF => {
                self.rom_bank = (self.rom_bank & 0x100) | value as usize;
                self.update_offsets();
            }
            0x3000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0xFF) | (((value & 1) as usize) << 8);
                self.update_offsets();
            }
            0x4000..=0x5FFF => {
                self.ram_bank = (value as usize) & 0x0F;
                self.update_offsets();
            }
            0xA000..=0xBFFF => self.write_ram(address as usize - 0xA000, value),
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.rom_bank = 1;
        self.ram_bank = 0;
        self.ram_enabled = false;
        self.update_offsets();
    }
}

pub trait Cartridge: MemoryMapped {
    fn cartridge_type(&self) -> CartridgeType;
    fn header(&self) -> &CartridgeHeader;
    fn read_abs(&self, address: usize) -> u8;
}

pub fn load_cartridge(filename: String) -> Box<dyn Cartridge> {
    let mut file = File::open(filename).unwrap();
    let mut content: Vec<u8> = Vec::new();
    file.read_to_end(&mut content).unwrap();

    let code = content[0x147];
    let cartridge_type = CartridgeType::from_rom(&content);

    return match cartridge_type {
        None => panic!("Unsupported cartridge type: 0x{:02x}", code),
        Some(t) => {
            println!("Cartridge type 0x{:02x}: {}", code, t.to_string());
            match t {
                CartridgeType::NoMBC { .. } => Box::new(NoMBC::new(t, &content)),
                CartridgeType::MBC1 { .. } => Box::new(MBC1::new(t, &content)),
                CartridgeType::MBC2 { .. } => Box::new(MBC2::new(t, &content)),
                CartridgeType::MBC5 { .. } => Box::new(MBC5::new(t, &content)),
                _ => panic!("Unsupported cartridge type: 0x{:02x}", code),
            }
        }
    };
}
