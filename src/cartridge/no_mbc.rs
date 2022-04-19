use super::{
    cartridge::Cartridge, cartridge_header::CartridgeHeader, cartridge_type::CartridgeType,
};
use crate::mmu::MemoryMapped;

pub struct NoMBC {
    // Memory buffers
    pub rom: Box<[u8]>,
    pub ram: Option<Box<[u8]>>,
    cartridge_type: CartridgeType,
    header: CartridgeHeader,
}

impl NoMBC {
    pub fn new(cartridge_type: CartridgeType, data: &Vec<u8>) -> Self {
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
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x7FFF => self.rom[address],
            0xA000..=0xBFFF => match &self.ram {
                Some(ram) => ram[address],
                None => 0xFF,
            },
            _ => 0xFF,
        }
    }

    fn write(&mut self, address: usize, value: u8) {
        match address {
            0xA000..=0xBFFF => {
                if let Some(ref mut ram) = self.ram {
                    ram[address - 0xA000] = value;
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
