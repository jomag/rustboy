use crate::cartridge::is_mbc1_multicart;
use crate::utils::VecExt;

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
    pub fn from_rom(rom: &Vec<u8>) -> Option<CartridgeType> {
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

    #[allow(dead_code)]
    fn has_rtc(&self) -> bool {
        use self::CartridgeType::*;
        match self {
            MBC3 { rtc, .. } => *rtc,
            _ => false,
        }
    }
}
