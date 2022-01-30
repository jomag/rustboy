use std::fs::File;
use std::io::Read;

pub trait Cartridge {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn reset(&mut self);
}

pub fn cartridge_type_name(cartridge_type: u8) -> String {
    match cartridge_type {
        0x00 => "ROM Only",
        0x01 => "MBC1",
        0x02 => "MBC1 with RAM",
        0x03 => "MBC1 with RAM and battery",
        0x05 => "MBC2",
        0x06 => "MBC2 with battery",
        0x08 => "ROM and RAM",
        0x09 => "ROM, RAM and battery",
        0x0b => "MMM01",
        0x0c => "MMM01 with RAM",
        0x0d => "MMM01 with RAM and battery",
        0x0f => "MBC3 with timer and battery",
        0x10 => "MBC3 with timer, RAM and battery",
        0x11 => "MBC3",
        0x12 => "MBC3 with RAM and battery",
        0x19 => "MBC5",
        0x1a => "MBC5 with RAM",
        0x1b => "MBC5 with RAM and battery",
        0x1c => "MBC5 with rumble",
        0x1d => "MBC5 with RAM and rumble",
        0x1e => "MBC5 with RAM, rumble and battery",
        0x20 => "MBC6",
        0x22 => "MBC7 with RAM, sensor, rumble and battery",
        0xfc => "Pocket camera",
        0xfd => "Bandai TAMA5",
        0xfe => "HuC3",
        0xff => "HuC1 with RAM and battery",
        _ => "unknown type",
    }
    .to_string()
}

struct CartridgeMBC1 {
    // MBC1 cartridges can have different sizes and the size
    // affects how banks are wrapped around, etc.
    // We set the size from the size of the cartridge ROM.
    pub size: usize,

    // Cartridges of type MBC1 can hold 125 banks of 16k.
    // Three banks are reserved, which is the reason for
    // the odd number instead of 128.
    pub rom: Box<[u8]>,

    // 32k RAM
    pub ram: Box<[u8]>,

    // 5 LSB of the ROM bank
    pub rom_bank_lower: u8,

    // 2 bit register that selects RAM bank *or* upper two
    // bits of the ROM bank
    pub rom_ram_bank: u8,

    // The 1 bit register that selects ROM or RAM mode
    pub rom_ram_mode: u8,

    pub ram_enabled: bool,

    // True if cartridge has ram and/or battery
    // Type 1 has no ram or battery
    // Type 2 has ram
    // Type 3 has ram and battery
    #[allow(dead_code)]
    pub with_ram: bool,
    #[allow(dead_code)]
    pub with_battery: bool,
}

impl CartridgeMBC1 {
    pub fn new(data: Vec<u8>, with_ram: bool, with_battery: bool) -> Self {
        let mut rom = vec![0x00; 0x4000 * 128].into_boxed_slice();

        for (src, dst) in rom.iter_mut().zip(data.iter()) {
            *src = *dst
        }

        CartridgeMBC1 {
            size: data.len(),
            rom,
            ram: vec![0; 0x8000].into_boxed_slice(),

            // ROM bank is initialized to 1
            rom_bank_lower: 1,

            rom_ram_bank: 0,
            rom_ram_mode: 0,
            ram_enabled: false,
            with_ram,
            with_battery,
        }
    }

    fn ram_offs(&self) -> usize {
        return if self.rom_ram_mode == 0 {
            // ROM mode - only RAM bank 0 is accessible
            0
        } else {
            // RAM mode
            self.rom_ram_bank as usize * 0x2000
        };
    }

    fn rom_offs(&self) -> usize {
        let bank = if self.rom_ram_mode == 0 {
            // ROM mode - all ROM banks are accessible
            (self.rom_ram_bank << 5) | (self.rom_bank_lower & 0x1F)
        } else {
            // RAM mode - only ROM bank 0x01-0x1F are accessible
            self.rom_bank_lower & 0x1F
        } as usize;
        return bank * 0x4000;
    }
}

impl Cartridge for CartridgeMBC1 {
    fn reset(&mut self) {
        self.ram.fill(0);
        self.rom_bank_lower = 1;
        self.rom_ram_bank = 0;
        self.ram_enabled = false;
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => self.rom[self.rom_offs() + address as usize - 0x4000],
            0xA000..=0xBFFF => self.ram[self.ram_offs() + address as usize - 0xA000],
            _ => {
                println!("Read from unhandled cartridge address: {:04x}", address);
                return 0;
            }
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                // Any value with 0x0A in the lower four bits enables RAM.
                // All other values disables RAM.
                self.ram_enabled = value & 0x0F == 0x0A;
            }

            0x2000..=0x3FFF => {
                // Set lower 5 bits of ROM bank. The higher bits are discarded.
                // Bank 0 is unusable, so bank 1 will be selected instead.
                // Same for bank 0x20, 0x40 and 0x60 (0x21, 0x41 and 0x61)
                //
                // TODO: if the cartridge is smaller so that it does not use
                // all banks, then the bank should wrap around. See Pandocs.
                self.rom_bank_lower = value & 0x1F;
                if self.rom_bank_lower == 0 {
                    self.rom_bank_lower = 1;
                }
            }

            0x4000..=0x5FFF => {
                // 1 bit register that select ROM or RAM mode
                self.rom_ram_mode = value & 1;
            }

            0x6000..=0x7FFF => {
                // Two bit register that selects RAM bank *or* 2 MSB of ROM bank
                self.rom_ram_bank = value & 3;
            }

            0xA000..=0xBFFF => {
                self.ram[self.ram_offs() + address as usize - 0xA000] = value;
            }

            _ => println!("Unhandled write to ROM: {:04x} = {:02x}", address, value),
        }
    }
}

struct Cartridge32k {
    pub rom: Box<[u8]>,
}

impl Cartridge32k {
    pub fn new(data: Vec<u8>) -> Self {
        let mut rom = vec![0; 0x8000].into_boxed_slice();
        let bytes = &data[..data.len()];
        rom.copy_from_slice(bytes);
        Cartridge32k { rom }
    }
}

impl Cartridge for Cartridge32k {
    fn reset(&mut self) {}

    fn read(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    fn write(&mut self, _address: u16, _value: u8) {}
}

pub struct NullCartridge;

impl Cartridge for NullCartridge {
    fn reset(&mut self) {}

    fn read(&self, _address: u16) -> u8 {
        0
    }

    fn write(&mut self, _address: u16, _value: u8) {}
}

pub fn load_cartridge(filename: String) -> Box<dyn Cartridge> {
    let mut file = File::open(filename).unwrap();
    let mut rom: Vec<u8> = Vec::new();

    // Returns amount of bytes read and append the rebsult to the buffer
    file.read_to_end(&mut rom).unwrap();

    let cartridge_type = rom[0x147];
    println!(
        "Cartridge type {:02x}: {}",
        cartridge_type,
        cartridge_type_name(cartridge_type)
    );

    match cartridge_type {
        0 => return Box::new(Cartridge32k::new(rom)) as Box<dyn Cartridge>,
        1 => {
            let a = CartridgeMBC1::new(rom, false, false);
            let b = Box::new(a) as Box<dyn Cartridge>;
            return b;
        }
        2 => return Box::new(CartridgeMBC1::new(rom, true, false)) as Box<dyn Cartridge>,
        3 => return Box::new(CartridgeMBC1::new(rom, false, false)) as Box<dyn Cartridge>,
        _ => panic!("Unsupported cartridge type: {:02X}", cartridge_type),
    };
}
