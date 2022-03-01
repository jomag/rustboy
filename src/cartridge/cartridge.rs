use super::{cartridge_header::CartridgeHeader, cartridge_type::CartridgeType};

use crate::mmu::MemoryMapped;

pub trait Cartridge: MemoryMapped {
    fn cartridge_type(&self) -> CartridgeType;
    fn header(&self) -> &CartridgeHeader;
    fn read_abs(&self, address: usize) -> u8;
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
