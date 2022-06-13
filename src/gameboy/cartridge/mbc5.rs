use super::super::mmu::MemoryMapped;
use super::cartridge::Cartridge;
use super::cartridge_header::{CartridgeHeader, RAM_BANK_SIZE, ROM_BANK_SIZE};
use super::cartridge_type::CartridgeType;

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
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address],
            0x4000..=0x7FFF => self.rom[self.rom_offset_0x4000_0x7fff + address - 0x4000],
            0xA000..=0xBFFF => self.read_ram(address - 0xA000),
            _ => 0xFF,
        }
    }

    fn write(&mut self, address: usize, value: u8) {
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
