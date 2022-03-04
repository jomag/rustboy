pub mod cartridge;
pub mod cartridge_header;
pub mod cartridge_type;
pub mod mbc1;
pub mod mbc2;
pub mod mbc3;
pub mod mbc5;
pub mod no_mbc;

use std::fs::File;
use std::io::Read;

use crate::cartridge::mbc3::MBC3;

use super::cartridge::{
    cartridge::Cartridge, cartridge_type::CartridgeType, mbc1::MBC1, mbc2::MBC2, mbc5::MBC5,
    no_mbc::NoMBC,
};

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
                CartridgeType::MBC3 { .. } => Box::new(MBC3::new(t, &content)),
                CartridgeType::MBC5 { .. } => Box::new(MBC5::new(t, &content)),
                _ => panic!("Unsupported cartridge type: 0x{:02x}", code),
            }
        }
    };
}
