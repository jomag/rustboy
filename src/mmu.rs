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

    // ROM bank (0x0000 to 0x3FFF)
    pub rom: [u8; 0x4000],

    // Switchable ROM bank (0x4000 to 0x7FFF)
    pub romx: [u8; 0x4000],

    // External RAM in cartridge
    pub external_ram: [u8; 0x2000],

    // RAM bank (0xC000 to 0xCFFF)
    pub ram: [u8; 0x2000],

    // I/O registers (0xFF00 to 0xFFFF)
    // FIXME: these are used to allow emulator to progress.
    // When we support all registers this memory area can be removed.
    pub io_reg: [u8; 0x80],
    ie_reg: u8,

    // Internal RAM (0xFF80 to 0xFFFF)
    pub internal_ram: [u8; 0x7F],

    bootstrap: [u8; 0x100],
    pub bootstrap_mode: bool,
    pub watch_triggered: bool,

    pub timer: Timer,
    pub dma: DMA,
    pub lcd: LCD,

    pub display_updated: bool,
}

impl MMU {
    pub fn new() -> Self {
        MMU {
            reg: Registers::new(),
            rom: [0; 0x4000],
            romx: [0; 0x4000],
            external_ram: [0; 0x2000],
            ram: [0; 0x2000],
            io_reg: [0; 0x80],
            ie_reg: 0,
            internal_ram: [0; 0x7F],
            bootstrap: [0; 0x100],
            bootstrap_mode: true,
            watch_triggered: false,
            timer: Timer::new(),
            dma: DMA::new(),
            lcd: LCD::new(),
            display_updated: false,
        }
    }

    pub fn init(&mut self) {
        self.io_reg[0xFF00 & 0x7F] = 0xCF;
        self.io_reg[0xFF01 & 0x7F] = 0x00;
        self.io_reg[0xFF02 & 0x7F] = 0x7E;

        // Undocumented, but should be initialized to 0xFF
        self.io_reg[0xFF03 & 0x4F] = 0xFF;
    }

    pub fn wakeup_if_halted(&mut self) {
        if self.reg.halted {
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
            self.tick(4);
        }

        handle_interrupts(self);
    }

    pub fn tick(&mut self, cycles: u32) {
        self.timer.update(cycles);

        if self.lcd.update(cycles) {
            self.display_updated = true;
        }

        if !self.reg.halted {
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
    }

    pub fn load_bootstrap(&mut self, filename: &str) {
        // Open and read content of boot rom
        let mut f = File::open(filename).expect("failed to open boot rom");
        f.read(&mut self.bootstrap)
            .expect("failed to read content of boot rom");
    }

    pub fn load_cartridge(&mut self, filename: &str) {
        let mut f = File::open(filename).expect("failed to open cartridge rom");
        f.read(&mut self.rom)
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
        match addr {
            0x0000...0x00FF => {
                if self.bootstrap_mode {
                    self.bootstrap[addr as usize]
                } else {
                    self.rom[addr as usize]
                }
            }
            0x0100...0x3FFF => self.rom[addr as usize],
            0x4000...0x7FFF => self.romx[(addr & 0x3FFF) as usize],
            0x8000...0x9FFF => self.lcd.read_display_ram(addr),
            0xA000...0xBFFF => self.external_ram[(addr & 0x1FFF) as usize],
            0xC000...0xCFFF => self.ram[(addr - 0xC000) as usize], // RAM
            0xD000...0xDFFF => self.ram[(addr - 0xC000) as usize], // RAM (switchable on GBC)
            0xE000...0xFDFF => self.ram[(addr - 0xE000) as usize], // RAM echo
            0xFE00...0xFE9F => {
                if self.dma.is_active() {
                    0xFF
                } else {
                    self.dma.read(addr - 0xFE00)
                }
            }
            0xFEA0...0xFEFF => 0, // Unused. Not emulated yet.

            // Special registers in area 0xFF00 to 0xFFFF
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

            // Use self.io_reg for I/O registers that have not been implemented yet
            0xFF00...0xFF7F => self.io_reg[(addr & 0x7F) as usize],

            0xFF80...0xFFFE => self.internal_ram[(addr & 0x7F) as usize],

            IE_REG => self.ie_reg,
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
        match addr {
            0x0000...0x3FFF => {}
            0x4000...0x7FFF => {}
            0x8000...0x9FFF => self.lcd.write_display_ram(addr, value),
            0xA000...0xBFFF => self.external_ram[(addr & 0x1FFF) as usize] = value,
            0xC000...0xCFFF => self.ram[(addr - 0xC000) as usize] = value,
            0xD000...0xDFFF => self.ram[(addr - 0xC000) as usize] = value,
            0xE000...0xFDFF => self.ram[(addr - 0xE000) as usize] = value,
            0xFE00...0xFE9F => self.dma.write(addr - 0xFE00, value),
            0xFEA0...0xFEFF => {} // unused and not yet emulated

            0xFF10...0xFF26 => println!(
                "Unhanlded write to audio register: 0x{:04X}={:02X}",
                addr, value
            ),
            0xFF30...0xFF3F => println!(
                "Unhandled write to wave register 0x{:04X}={:02X}",
                addr, value
            ),

            P1_REG => {}
            SB_REG => {}
            SC_REG => {}
            DIV_REG => self.timer.write_div(value),
            TIMA_REG => self.timer.tima = value,
            TMA_REG => self.timer.tma = value,
            TAC_REG => self.timer.tac = value,
            0xFF08 => println!("write to 0xFF08 - undocumented!: {}", value),
            IF_REG => self.set_if_reg(value),

            LCDC_REG => self.lcd.lcdc = value,
            STAT_REG => self.lcd.set_stat_reg(value),
            SCY_REG => self.lcd.scy = value,
            SCX_REG => self.lcd.scx = value,
            LY_REG => self.lcd.scanline = value,
            LYC_REG => self.lcd.lyc = value,
            DMA_REG => self.dma.start(value),
            BGP_REG => {}
            OBP0_REG => {}
            OBP1_REG => {}
            WY_REG => {}
            WX_REG => {}

            0xFF4D => println!("write to 0xFF4D - KEY1 (CGB only): {}", value),

            // 0xFF50: write 1 to disable bootstrap ROM
            0xFF50 => self.bootstrap_mode = false,

            // Invalid registers, that are still used by for example Tetris
            // https://www.reddit.com/r/EmuDev/comments/5nixai/gb_tetris_writing_to_unused_memory/
            0xFF7F => {}

            0xFF00...0xFF7F => self.io_reg[(addr & 0xFF) as usize] = value,
            0xFF80...0xFFFE => self.internal_ram[(addr & 0x7F) as usize] = value,

            IE_REG => {
                println!("SET IE TO {}", value);
                self.ie_reg = value
            }
        };
    }

    pub fn pop(&mut self) -> u16 {
        let sp = self.reg.sp;
        let v = self.read_u16(sp);
        self.reg.sp = sp.wrapping_add(2);
        v
    }
}
