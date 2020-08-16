use std::fs::File;
use std::io::Read;

pub trait Cartridge {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

struct CartridgeMBC1 {
    // Cartridges of type MBC1 can hold 125 banks of 16k.
    // Three banks are reserved, which is the reason for
    // the odd number instead of 128.
    pub rom: [u8; 0x4000 * 128],

    // Current ROM offset, depending on bank selection
    pub rom_offset: usize,
}

impl CartridgeMBC1 {
    pub fn new(data: Vec<u8>) -> Self {
        let mut rom = [0; 0x4000 * 128];
        for (src, dst) in rom.iter_mut().zip(data.iter()) {
            *src = *dst
        }
        CartridgeMBC1 {
            rom: rom,
            rom_offset: 0,
        }
    }
}

impl Cartridge for CartridgeMBC1 {
    fn read(&self, address: u16) -> u8 {
        if address < 0x4000 {
            self.rom[address as usize]
        } else {
            self.rom[self.rom_offset + address as usize - 0x4000]
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x2000...0x3FFF => self.rom_offset = (value as usize) * 0x4000,
            _ => println!("Unhandled write to ROM: {:04x} = {:02x}", address, value),
        }
    }
}

struct Cartridge32k {
    pub rom: [u8; 0x8000],
}

impl Cartridge32k {
    pub fn new(data: Vec<u8>) -> Self {
        let mut rom = [0; 0x8000];
        let bytes = &data[..data.len()];
        rom.copy_from_slice(bytes);
        Cartridge32k { rom: rom }
    }
}

impl Cartridge for Cartridge32k {
    fn read(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {}
}

pub struct NullCartridge;

impl Cartridge for NullCartridge {
    fn read(&self, address: u16) -> u8 {
        0
    }

    fn write(&mut self, address: u16, value: u8) {}
}

pub fn load_cartridge(filename: &str) -> Box<Cartridge> {
    let mut file = File::open(filename).unwrap();
    let mut rom: Vec<u8> = Vec::new();

    // Returns amount of bytes read and append the result to the buffer
    let result = file.read_to_end(&mut rom).unwrap();
    println!("Read {} bytes", result);

    let cartridge_type = rom[0x147];
    println!("Cartridge type: {:02x}", cartridge_type);

    match cartridge_type {
        0 => return Box::new(Cartridge32k::new(rom)) as Box<Cartridge>,
        1 => return Box::new(CartridgeMBC1::new(rom)) as Box<Cartridge>,
        _ => panic!("Unsupported cartridge type: {:02X}", cartridge_type),
    }
}
