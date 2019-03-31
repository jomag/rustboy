extern crate ansi_term;

use mmu::ansi_term::Colour::Blue;
use std::fs::File;
use std::io::Read;

use interrupt::{IF_LCDC_BIT, IF_TMR_BIT, IF_VBLANK_BIT};

use dma::DMA;
use instructions;
use interrupt::handle_interrupts;
use lcd::LCD;
use registers::Registers;
use timer::Timer;

use debug::print_registers;

// Port/Mode registers
pub const P1_REG: u16 = 0xFF00;
pub const SB_REG: u16 = 0xFF01;
pub const SC_REG: u16 = 0xFF02;
pub const DIV_REG: u16 = 0xFF04;
pub const TIMA_REG: u16 = 0xFF05; // timer counter
pub const TMA_REG: u16 = 0xFF06; // timer modulo
pub const TAC_REG: u16 = 0xFF07; // timer control

// Interrupt Flags
pub const IF_REG: u16 = 0xFF0F;
pub const IE_REG: u16 = 0xFFFF;

// LCD registers
pub const LCDC_REG: u16 = 0xFF40;
pub const STAT_REG: u16 = 0xFF41;
pub const SCY_REG: u16 = 0xFF42;
pub const SCX_REG: u16 = 0xFF43;
pub const LY_REG: u16 = 0xFF44;
pub const LYC_REG: u16 = 0xFF45;
pub const DMA_REG: u16 = 0xFF46;
pub const BGP_REG: u16 = 0xFF47;
pub const OBP0_REG: u16 = 0xFF48;
pub const OBP1_REG: u16 = 0xFF49;
pub const WY_REG: u16 = 0xFF4A;
pub const WX_REG: u16 = 0xFF4B;

// Memory areas
pub const OAM_OFFSET: u16 = 0xFE00;

pub struct MMU {
    pub reg: Registers,
    pub mem: [u8; 0x10000],
    bootstrap: [u8; 0x100],
    pub bootstrap_mode: bool,
    pub watch_triggered: bool,

    pub timer: Timer,
    pub dma: DMA,
    pub lcd: LCD,

    pub display_updated: bool,
    pub halted: bool,
}

// impl Serialize for MMU {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut state = serializer.serialize_struct("MMU", 2)?;
//         state.serialize_field("reg", &self.reg);

//     }
// }

impl MMU {
    pub fn new() -> Self {
        MMU {
            reg: Registers::new(),
            mem: [0; 0x10000],
            bootstrap: [0; 0x100],
            bootstrap_mode: true,
            watch_triggered: false,
            timer: Timer::new(),
            dma: DMA::new(),
            lcd: LCD::new(),
            display_updated: false,
            halted: false,
        }
    }

    pub fn init(&mut self) {
        self.mem[0xFF00] = 0xCF;
        self.mem[0xFF01] = 0x00;
        self.mem[0xFF02] = 0x7E;

        // Undocumented, but should be initialized to 0xFF
        self.mem[0xFF03] = 0xFF;
    }

    pub fn wakeup_if_halted(&mut self) {
        if self.reg.halted {
            println!("unhalted!");
            self.reg.halted = false;
        }
    }

    pub fn get_if_reg(&self) -> u8 {
        return self.lcd.irq | self.timer.irq;
    }

    pub fn set_if_reg(&mut self, value: u8) {
        self.lcd.irq = value & (IF_VBLANK_BIT | IF_LCDC_BIT);
        self.timer.irq = value & IF_TMR_BIT;
    }

    pub fn clear_if_reg_bits(&mut self, mask: u8) {
        self.lcd.irq &= !mask;
        self.timer.irq &= !mask;
    }

    pub fn exec_op(&mut self) {
        if !self.reg.halted {
            instructions::step(self);
        } else {
            self.tick(1);
        }

        handle_interrupts(self);
    }

    pub fn tick(&mut self, cycles: u32) {
        self.timer.update(cycles);

        if self.lcd.update(cycles, &mut self.mem) {
            self.display_updated = true;
        }

        // FIXME: Handle cycles not divisible by 4,
        // for example while halted as the machine
        // steps cycle by cycle in that mode.
        // We could do it by adding a cycle counter
        // to self.dma, and proceed with dma update
        // every 4:th cycle
        for _ in 0..(cycles / 4) {
            if self.dma.is_active() {
                let offset = self.dma.start_address.unwrap();
                let idx = self.dma.step;
                let b = self.direct_read(offset + idx);
                self.dma.oam[idx as usize] = b;
                println!(
                    "DMA stuff.. from {:x} to {:x}",
                    offset + idx,
                    OAM_OFFSET + idx
                );
            }
            self.dma.update();
        }
    }

    pub fn load_bootstrap(&mut self, filename: &str) {
        // Open and read content of boot rom
        let mut f = File::open(filename).expect("failed to open boot rom");
        f.read(&mut self.bootstrap)
            .expect("failed to read content of boot rom");
    }

    pub fn load_cartridge(&mut self, filename: &str) {
        let mut f = File::open(filename).expect("failed to open cartridge rom");
        f.read(&mut self.mem)
            .expect("failed to read content of cartridge rom");
    }

    pub fn fetch(&mut self) -> u8 {
        let pc = self.reg.pc;
        let value = self.read(pc);
        self.reg.pc = pc.wrapping_add(1);
        value
    }

    pub fn fetch_u16(&mut self) -> u16 {
        let lo = self.fetch();
        let hi = self.fetch();
        return ((hi as u16) << 8) | (lo as u16);
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        self.tick(4);
        self.direct_read(addr)
    }

    pub fn direct_read(&self, addr: u16) -> u8 {
        if addr < 0x100 && self.bootstrap_mode {
            return self.bootstrap[addr as usize];
        } else if addr >= 0x8000 && addr < 0xA000 {
            return self.lcd.read_display_ram(addr);
        } else if addr >= 0xFE00 && addr < 0xFEA0 {
            return self.dma.read(addr - 0xFE00);
        } else {
            match addr {
                IF_REG => self.get_if_reg(),
                DIV_REG => self.timer.read_div(),
                TIMA_REG => self.timer.tima,
                TMA_REG => self.timer.tma,
                TAC_REG => self.timer.tac,

                LCDC_REG => self.lcd.lcdc,
                STAT_REG => self.lcd.get_stat_reg(),
                SCY_REG => self.lcd.scy,
                SCX_REG => self.lcd.scx,
                LY_REG => self.lcd.scanline,
                LYC_REG => self.lcd.lyc,

                DMA_REG => self.dma.last_write_dma_reg,

                _ => self.mem[addr as usize],
            }
        }
    }

    pub fn read_i8(&mut self, addr: u16) -> i8 {
        let v = self.read(addr);
        return (0 as i8).wrapping_add(v as i8);
    }

    pub fn direct_read_i8(&self, addr: u16) -> i8 {
        let v = self.direct_read(addr);
        return (0 as i8).wrapping_add(v as i8);
    }

    pub fn read_u16(&mut self, addr: u16) -> u16 {
        let lo = self.read(addr);
        let hi = self.read(addr + 1);
        return ((hi as u16) << 8) | (lo as u16);
    }

    pub fn direct_read_u16(&self, addr: u16) -> u16 {
        let lo = self.direct_read(addr);
        let hi = self.direct_read(addr + 1);
        return ((hi as u16) << 8) | (lo as u16);
    }

    pub fn write_u16(&mut self, addr: u16, value: u16) {
        self.write(addr, (value & 0xFF) as u8);
        self.write(addr + 1, ((value >> 8) & 0xFF) as u8);
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.tick(4);
        self.direct_write(addr, value)
    }

    pub fn direct_write(&mut self, addr: u16, value: u8) {
        //if addr >= 0xD000 && addr < 0xD100 {
        //    println!("Write to watched memory location 0x{:04X}. Current: 0x{:02X}. New value: 0x{:02X}", addr, self.mem[addr as usize], value);
        //    self.watch_triggered = true;
        //}

        // println!("WRITE MEM: 0x{:04X} = 0x{:02X} ({})", addr, value, address_type(addr));
        if addr >= 0x8000 && addr < 0xA000 {
            self.lcd.write_display_ram(addr, value)
        } else if addr >= 0xFE00 && addr < 0xFEA0 {
            self.dma.write(addr - 0xFE00, value);
        } else if addr == 0xFF0F {
            self.set_if_reg(value)
        } else if addr == 0xFFFF {
            // println!("Write to IE register 0xFFFF: {}", value);
        } else if addr >= 0xFF80 && addr <= 0xFFFE {

        } else if addr >= 0xFF00 {
            if addr >= 0xFF10 && addr <= 0xFF26 {
                println!(
                    "unhandled write to audio register 0x{:04X}: {}",
                    addr, value
                );
            } else if addr >= 0xFF30 && addr <= 0xFF3F {
                println!("unhandled write to wave register 0x{:04X}: {}", addr, value);
            } else {
                match addr {
                    0xFF00 => {} // P1
                    0xFF01 => {}
                    0xFF02 => {
                        if value == 0x81 {
                            let s = format!("{}", self.mem[0xFF01] as char);
                            print!("{}", Blue.bold().paint(s)) // SB
                        }
                    }
                    0xFF04 => self.timer.write_div(value),
                    0xFF05 => self.timer.tima = value, // TIMA
                    0xFF06 => self.timer.tma = value,  // TMA
                    0xFF07 => self.timer.tac = value,  // TAC
                    0xFF08 => println!("write to 0xFF08 - undocumented!: {}", value),

                    0xFF40 => self.lcd.lcdc = value,
                    0xFF41 => self.lcd.set_stat_reg(value), // STAT
                    0xFF42 => self.lcd.scy = value,
                    0xFF43 => self.lcd.scx = value, // SCX
                    0xFF44 => self.lcd.scanline = value,
                    0xFF45 => self.lcd.lyc = value,
                    0xFF46 => self.dma.start(value), // DMA
                    0xFF47 => {}                     // BGP
                    0xFF48 => {}                     // OBP0
                    0xFF49 => {}                     // OBP1
                    0xFF4A => {}                     // WY
                    0xFF4B => {}                     // WX
                    0xFF4D => println!("write to 0xFF4D - KEY1 (CGB only): {}", value),

                    // Invalid registers, that are still used by for example Tetris
                    // https://www.reddit.com/r/EmuDev/comments/5nixai/gb_tetris_writing_to_unused_memory/
                    0xFF7F => {}

                    // 0xFF50: write 1 to disable bootstrap ROM
                    0xFF50 => self.bootstrap_mode = false,
                    _ => panic!(
                        "unhandled write to special register 0x{:04X}: {}",
                        addr, value
                    ),
                }
            }
        }

        self.mem[addr as usize] = value;
    }

    pub fn pop(&mut self) -> u16 {
        let sp = self.reg.sp;
        let v = self.read_u16(sp);
        self.reg.sp = sp.wrapping_add(2);
        v
    }
}
