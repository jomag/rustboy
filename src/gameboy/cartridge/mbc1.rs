use super::super::mmu::MemoryMapped;
use crate::conv;

use super::{
    cartridge::Cartridge, cartridge_header::CartridgeHeader, cartridge_type::CartridgeType,
};

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

impl MBC1 {
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
            header,
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
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[self.rom_offset_0x0000_0x3fff + address],
            0x4000..=0x7FFF => self.rom[self.rom_offset_0x4000_0x7fff + address - 0x4000],
            0xA000..=0xBFFF => self.read_ram(address - 0xA000),
            _ => 0,
        }
    }

    fn write(&mut self, address: usize, value: u8) {
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
