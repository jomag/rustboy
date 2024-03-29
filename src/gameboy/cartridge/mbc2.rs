use super::super::mmu::MemoryMapped;
use super::{
    cartridge::Cartridge, cartridge_header::CartridgeHeader, cartridge_type::CartridgeType,
};

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
    pub fn new(cartridge_type: CartridgeType, data: &Vec<u8>) -> Self {
        let header = CartridgeHeader::from_header(data);

        let mut rom = vec![0; header.rom_size].into_boxed_slice();
        for (src, dst) in rom.iter_mut().zip(data.iter()) {
            *src = *dst
        }

        // Note: the RAM size reported in the header seems to be 0 bytes,
        // but MBC2 always have 512 x 4 bit RAM.
        let ram = vec![0; cartridge_type.max_ram_size()].into_boxed_slice();

        let mut cartridge = MBC2 {
            rom,
            ram,
            ram_enabled: false,
            bank: 1,
            rom_offset_0x4000_0x7fff: 0,
            cartridge_type,
            header,
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
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address],
            0x4000..=0x7FFF => self.rom[self.rom_offset_0x4000_0x7fff + address - 0x4000],
            0xA000..=0xBFFF => match self.ram_enabled {
                true => self.ram[(address - 0xA000) & 0x1ff] | 0xF0,
                false => 0xFF,
            },
            _ => 0,
        }
    }

    fn write(&mut self, address: usize, value: u8) {
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
                    self.ram[(address - 0xA000) & 0x1ff] = value;
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
