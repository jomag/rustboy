
extern crate ansi_term;

use std::io::Read;
use std::fs::File;
use memory::ansi_term::Colour::Blue;

use debug::address_type;
use timer::Timer;

// Port/Mode registers
pub const P1_REG:   u16 = 0xFF00;
pub const SB_REG:   u16 = 0xFF01;
pub const SC_REG:   u16 = 0xFF02;
pub const DIV_REG:  u16 = 0xFF04;
pub const TIMA_REG: u16 = 0xFF05;  // timer counter
pub const TMA_REG:  u16 = 0xFF06;  // timer modulo
pub const TAC_REG:  u16 = 0xFF07;  // timer control

// Interrupt Flags
pub const IF_REG: u16 = 0xFF0F;
pub const IE_REG: u16 = 0xFFFF;

// LCD registers
pub const LCDC_REG: u16 = 0xFF40;
pub const STAT_REG: u16 = 0xFF41;
pub const SCY_REG:  u16 = 0xFF42;
pub const SCX_REG:  u16 = 0xFF43;
pub const LY_REG:   u16 = 0xFF44;
pub const LYC_REG:  u16 = 0xFF45;
pub const DMA_REG:  u16 = 0xFF46;
pub const BGP_REG:  u16 = 0xFF47;
pub const OBP0_REG: u16 = 0xFF48;
pub const OBP1_REG: u16 = 0xFF49;
pub const WY_REG:   u16 = 0xFF4A;
pub const WX_REG:   u16 = 0xFF4B;

 

pub struct Memory {
    pub mem: [u8; 0x10000],
    bootstrap: [u8; 0x100],
    pub bootstrap_mode: bool,
    pub watch_triggered: bool,
    pub timer: Timer
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            mem: [0; 0x10000],
            bootstrap: [0; 0x100],
            bootstrap_mode: true,
            watch_triggered: false,
            timer: Timer::new()
        }
    }

    pub fn load_bootstrap(&mut self, filename: &str) {
        // Open and read content of boot rom
        let mut f = File::open(filename)
            .expect("failed to open boot rom");
        f.read(&mut self.bootstrap)
            .expect("failed to read content of boot rom");
    }

    pub fn load_cartridge(&mut self, filename: &str) {
        let mut f = File::open(filename)
            .expect("failed to open cartridge rom");
        f.read(&mut self.mem)
            .expect("failed to read content of cartridge rom");
    }

    pub fn read(&self, addr: u16) -> u8 {
        // Read byte (u8) from memory

        // if addr >= 0xFF00 {
        //     println!("READ MEM: 0x{:04X} ({})", addr, address_type(addr));
        // }

        if addr < 0x100 && self.bootstrap_mode {
            return self.bootstrap[addr as usize];
        } else {
            match addr {
            DIV_REG => { self.timer.read_div() }
            TIMA_REG => { self.timer.tima }
            TMA_REG => { self.timer.tma }
            TAC_REG => { self.timer.tac }
            _ => { self.mem[addr as usize] }
            }
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

    pub fn write_u16(&mut self, addr: u16, value: u16) {
        self.write(addr, (value & 0xFF) as u8);
        self.write(addr + 1, ((value >> 8) & 0xFF) as u8);
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        //if addr >= 0xD000 && addr < 0xD100 {
        //    println!("Write to watched memory location 0x{:04X}. Current: 0x{:02X}. New value: 0x{:02X}", addr, self.mem[addr as usize], value);
        //    self.watch_triggered = true;
        //}

        // println!("WRITE MEM: 0x{:04X} = 0x{:02X} ({})", addr, value, address_type(addr));
        if addr == 0xFF0F {
            // println!("Write to IF register 0xFF0F: {}", value);
        }
        
        else if addr == 0xFFFF {
            // println!("Write to IE register 0xFFFF: {}", value);
        }
        
        else if addr >= 0xFF80 && addr <= 0xFFFE {

        } else if addr >= 0xFF00 {
            if addr >= 0xFF10 && addr <= 0xFF26 {
                println!("unhandled write to audio register 0x{:04X}: {}", addr, value);
            } else if addr >= 0xFF30 && addr <= 0xFF3F {
                println!("unhandled write to wave register 0x{:04X}: {}", addr, value);
            } else {
                match addr {
                    0xFF00 => {}  // P1
                    0xFF01 => {}
                    0xFF02 => {
                        if value == 0x81 {
                            let s = format!("{}", self.mem[0xFF01] as char);
                            print!("{}", Blue.bold().paint(s))  // SB
                        }
                    }
                    0xFF04 => { self.timer.write_div(value) }
                    0xFF05 => { self.timer.tima = value }  // TIMA
                    0xFF06 => { self.timer.tma = value }  // TMA
                    0xFF07 => { self.timer.tac = value }  // TAC
                    0xFF08 => { println!("write to 0xFF08 - undocumented!: {}", value) }

                    0xFF40 => {}
                    0xFF41 => {}  // STAT
                    0xFF42 => {}
                    0xFF43 => {}  // SCX
                    0xFF46 => {}  // DMA
                    0xFF47 => {}  // BGP
                    0xFF48 => {}  // OBP0
                    0xFF49 => {}  // OBP1
                    0xFF4A => {}  // WY
                    0xFF4B => {}  // WX
                    0xFF4D => { println!("write to 0xFF4D - KEY1 (CGB only): {}", value) }

                    // Invalid registers, that are still used by for example Tetris
                    // https://www.reddit.com/r/EmuDev/comments/5nixai/gb_tetris_writing_to_unused_memory/
                    0xFF7F => {}

                    // 0xFF50: write 1 to disable bootstrap ROM
                    0xFF50 => { self.bootstrap_mode = false }
                    _ => { panic!("unhandled write to special register 0x{:04X}: {}", addr, value) }
                }
            }
        }

        self.mem[addr as usize] = value;
    }
}

