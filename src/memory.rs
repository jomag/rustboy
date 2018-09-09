
use std::io::Read;
use std::fs::File;

use debug::address_type;

pub struct Memory {
    mem: [u8; 0x10000],
    bootstrap: [u8; 0x100]
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            mem: [0; 0x10000],
            bootstrap: [0; 0x100]
        }
    }

    pub fn load_bootstrap(&mut self, filename: &str) {
        // Open and read content of boot rom
        let mut f = File::open(filename)
            .expect("failed to open boot rom");
        f.read(&mut self.bootstrap)
            .expect("failed to read content of boot rom");
    }

    pub fn read(&self, addr: u16) -> u8 {
        // Read byte (u8) from memory
        if addr < 0x100 {
            return self.bootstrap[addr as usize];
        } else {
            return self.mem[addr as usize];
        }
    }

    pub fn read_i8(&self, addr: u16) -> i8 {
        let v = self.read(addr);
        return (0 as i8).wrapping_add(v as i8);
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let lo = self.read(addr);
        let hi = self.read(addr + 1);
        return ((hi as u16) << 8) | (lo as u16);
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        println!("WRITE MEM: 0x{:04X} = 0x{:02X} ({})", addr, value, address_type(addr));
        self.mem[addr as usize] = value;
    }
}

