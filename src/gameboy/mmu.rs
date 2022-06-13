extern crate ansi_term;

use std::fs::File;
use std::io::Read;

use super::emu::Machine;
use super::interrupt::{IF_INP_BIT, IF_LCDC_BIT, IF_TMR_BIT, IF_VBLANK_BIT};

use super::apu::apu::{AudioProcessingUnit, SAMPLES_PER_FRAME};
use super::buttons::Buttons;
use super::cartridge::{cartridge::Cartridge, cartridge::NoCartridge, load_cartridge};
use super::dma::DMA;
use super::instructions;
use super::interrupt::handle_interrupts;
use super::ppu::PPU;
use super::registers::Registers;
use super::serial::Serial;
use super::timer::Timer;

pub const OAM_OFFSET: usize = 0xFE00;

// Port/Mode registers
pub const P1_REG: usize = 0xFF00;
pub const SB_REG: usize = 0xFF01; // serial transfer data
pub const SC_REG: usize = 0xFF02; // serial transfer control
pub const DIV_REG: usize = 0xFF04;
pub const TIMA_REG: usize = 0xFF05; // timer counter
pub const TMA_REG: usize = 0xFF06; // timer modulo
pub const TAC_REG: usize = 0xFF07; // timer control

// Interrupt Flags
pub const IF_REG: usize = 0xFF0F;
pub const IE_REG: usize = 0xFFFF;

// LCD registers
pub const LCDC_REG: usize = 0xFF40;
pub const STAT_REG: usize = 0xFF41;
pub const SCY_REG: usize = 0xFF42;
pub const SCX_REG: usize = 0xFF43;
pub const LY_REG: usize = 0xFF44;
pub const LYC_REG: usize = 0xFF45;
pub const DMA_REG: usize = 0xFF46;
pub const BGP_REG: usize = 0xFF47;
pub const OBP0_REG: usize = 0xFF48;
pub const OBP1_REG: usize = 0xFF49;
pub const WY_REG: usize = 0xFF4A;
pub const WX_REG: usize = 0xFF4B;

// Sound registers
// - Sound Generator 1
pub const NR10_REG: usize = 0xFF10;
pub const NR11_REG: usize = 0xFF11;
pub const NR12_REG: usize = 0xFF12;
pub const NR13_REG: usize = 0xFF13;
pub const NR14_REG: usize = 0xFF14;
// - Sound Generator 2
pub const NR20_REG: usize = 0xFF15;
pub const NR21_REG: usize = 0xFF16;
pub const NR22_REG: usize = 0xFF17;
pub const NR23_REG: usize = 0xFF18;
pub const NR24_REG: usize = 0xFF19;
// - Sound Generator 3
pub const NR30_REG: usize = 0xFF1A;
pub const NR31_REG: usize = 0xFF1B;
pub const NR32_REG: usize = 0xFF1C;
pub const NR33_REG: usize = 0xFF1D;
pub const NR34_REG: usize = 0xFF1E;
// - Sound Generator 4
pub const NR40_REG: usize = 0xFF1F;
pub const NR41_REG: usize = 0xFF20;
pub const NR42_REG: usize = 0xFF21;
pub const NR43_REG: usize = 0xFF22;
pub const NR44_REG: usize = 0xFF23;
// - Sound Control
pub const NR50_REG: usize = 0xFF24;
pub const NR51_REG: usize = 0xFF25;
pub const NR52_REG: usize = 0xFF26;

// FIXME: Same as MemoryMapped, but using u16 instead of usize.
//        All code should be updated to use MemoryMapped instead.
pub trait MemoryMapped16 {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);

    // Perform reset as after power cycle
    fn reset(&mut self);
}

pub trait MemoryMapped {
    fn read(&self, address: usize) -> u8;
    fn write(&mut self, address: usize, value: u8);

    // Perform reset as after power cycle
    fn reset(&mut self);
}

pub struct MMU {
    pub reg: Registers,
    pub cartridge: Box<dyn Cartridge>,

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
    pub ppu: PPU,
    pub buttons: Buttons,
    pub apu: AudioProcessingUnit,
    pub serial: Serial,

    pub display_updated: bool,

    // If interrupt handler was entered while executing
    // the previous operation, this variable is set to
    // the interrupt bit. Otherwise it's reset to zero.
    pub entered_interrupt_handler: u8,

    pub sample_count: u32,
}

impl MMU {
    pub fn new(machine: Machine) -> Self {
        MMU {
            reg: Registers::new(),
            cartridge: Box::new(NoCartridge {}),
            ram: [0; 0x2000],
            io_reg: [0; 0x80],
            ie_reg: 0,
            internal_ram: [0; 0x7F],
            bootstrap: [0; 0x100],
            bootstrap_mode: true,
            watch_triggered: false,
            timer: Timer::new(),
            dma: DMA::new(),
            ppu: PPU::new(machine),
            buttons: Buttons::new(),
            display_updated: false,
            entered_interrupt_handler: 0,

            // Create APU that will buffer up to 10 frames of audio
            apu: AudioProcessingUnit::new(machine, SAMPLES_PER_FRAME as u32 * 10),

            sample_count: 0,
            serial: Serial::new(None),
        }
    }

    pub fn reset(&mut self) {
        self.reg = Registers::new();
        self.cartridge.reset();
        self.ram.fill(0);
        self.io_reg.fill(0);
        self.ie_reg = 0;
        self.internal_ram.fill(0);
        self.bootstrap_mode = true;
        self.watch_triggered = false;
        self.timer = Timer::new();
        self.dma = DMA::new();
        self.ppu.reset();
        self.buttons = Buttons::new();
        self.display_updated = false;
        self.entered_interrupt_handler = 0;

        // The APU shares a ringbuf with audio code so it can't be recreated
        self.apu.reset();

        self.serial = Serial::new(None);
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
        return self.ppu.irq | self.timer.irq | self.buttons.irq;
    }

    pub fn set_if_reg(&mut self, value: u8) {
        self.ppu.irq = value & (IF_VBLANK_BIT | IF_LCDC_BIT);
        self.timer.irq = value & IF_TMR_BIT;
        self.buttons.irq = value & IF_INP_BIT;
    }

    pub fn clear_if_reg_bits(&mut self, mask: u8) {
        self.ppu.irq &= !mask;
        self.timer.irq &= !mask;
        self.buttons.irq &= !mask;
    }

    pub fn exec_op(&mut self) {
        if !self.reg.halted {
            instructions::step(self);
        } else {
            self.tick(4);
        }

        self.entered_interrupt_handler = handle_interrupts(self);
    }

    pub fn tick(&mut self, cycles: u32) {
        assert!(cycles % 4 == 0);

        for _ in 0..cycles / 4 {
            self.timer.update_4t();
            self.apu.update_4t(self.timer.cycle);
        }

        self.buttons.update();

        let updated = self.ppu.update(cycles);
        self.display_updated = self.display_updated || updated;

        if !self.reg.halted {
            for _ in 0..(cycles / 4) {
                if self.dma.is_active() {
                    let offset = self.dma.start_address.unwrap() as usize;
                    let idx = self.dma.step as usize;
                    let b = if offset < 0xE000 {
                        self.direct_read(offset + idx)
                    } else {
                        self.ram[(offset + idx - 0xE000) as usize]
                    };
                    self.ppu.write(OAM_OFFSET + idx, b)
                }
                self.dma.update();
            }
        }
    }

    pub fn load_bootstrap(&mut self, filename: &str) -> usize {
        // Open and read content of boot rom
        let mut f = File::open(filename).expect("failed to open boot rom");
        f.read(&mut self.bootstrap)
            .expect("failed to read content of boot rom")
    }

    pub fn load_cartridge(&mut self, filename: &str) {
        self.cartridge = load_cartridge(filename.to_string());
    }

    pub fn fetch(&mut self) -> u8 {
        let pc = self.reg.pc;
        let value = self.read(pc as usize);
        self.reg.pc = pc.wrapping_add(1);
        value
    }

    pub fn fetch_u16(&mut self) -> u16 {
        let lo = self.fetch();
        let hi = self.fetch();
        return ((hi as u16) << 8) | (lo as u16);
    }

    pub fn read(&mut self, addr: usize) -> u8 {
        self.tick(4);
        self.direct_read(addr)
    }

    pub fn direct_read(&self, addr: usize) -> u8 {
        match addr {
            0x0000..=0x00FF => {
                if self.bootstrap_mode {
                    self.bootstrap[addr as usize]
                } else {
                    self.cartridge.read(addr)
                }
            }
            0x0100..=0x3FFF => self.cartridge.read(addr),
            0x4000..=0x7FFF => self.cartridge.read(addr), // self.romx[(addr - 0x4000) as usize],
            0x8000..=0x9FFF => self.ppu.read(addr),
            0xA000..=0xBFFF => self.cartridge.read(addr),
            0xC000..=0xCFFF => self.ram[(addr - 0xC000)], // RAM
            0xD000..=0xDFFF => self.ram[(addr - 0xC000)], // RAM (switchable on GBC)
            0xE000..=0xFDFF => self.ram[(addr - 0xE000)], // RAM echo
            0xFE00..=0xFE9F => self.ppu.read(addr),

            // Unused and undocumented area. Seems to return 0 on DMG, and
            // random values on CGB.
            0xFEA0..=0xFEFF => 0x00,

            // Special registers in area 0xFF00 to 0xFFFF
            SB_REG..=SC_REG => self.serial.read_reg(addr),
            P1_REG => self.buttons.read_p1(),
            IF_REG => self.get_if_reg(),
            DIV_REG => self.timer.read_div(),
            TIMA_REG => self.timer.tima,
            TMA_REG => self.timer.tma,
            TAC_REG => self.timer.tac,
            LCDC_REG => self.ppu.read(addr),
            STAT_REG => self.ppu.read(addr),
            SCY_REG => self.ppu.read(addr),
            SCX_REG => self.ppu.read(addr),
            LY_REG => self.ppu.read(addr),
            LYC_REG => self.ppu.read(addr),
            DMA_REG => self.dma.last_write_dma_reg,
            BGP_REG => self.ppu.read(addr),
            OBP0_REG => self.ppu.read(addr),
            OBP1_REG => self.ppu.read(addr),
            WX_REG => self.ppu.read(addr),
            WY_REG => self.ppu.read(addr),

            // Sound registers
            0xFF10..=0xFF3F => self.apu.read_reg(addr),

            // Use self.io_reg for I/O registers that have not been implemented yet
            0xFF00..=0xFF7F => self.io_reg[(addr - 0xFF00) as usize],

            0xFF80..=0xFFFE => self.internal_ram[(addr - 0xFF80) as usize],

            IE_REG => self.ie_reg,

            _ => panic!("Read of unhandled address: 0x{:x}", addr),
        }
    }

    #[allow(dead_code)]
    pub fn read_i8(&mut self, addr: usize) -> i8 {
        let v = self.read(addr);
        return (0 as i8).wrapping_add(v as i8);
    }

    pub fn direct_read_i8(&self, addr: usize) -> i8 {
        let v = self.direct_read(addr);
        return (0 as i8).wrapping_add(v as i8);
    }

    #[allow(dead_code)]
    pub fn read_u16(&mut self, addr: usize) -> u16 {
        let lo = self.read(addr);
        let hi = self.read(addr + 1);
        return ((hi as u16) << 8) | (lo as u16);
    }

    pub fn direct_read_u16(&self, addr: usize) -> u16 {
        let lo = self.direct_read(addr);
        let hi = self.direct_read(addr + 1);
        return ((hi as u16) << 8) | (lo as u16);
    }

    pub fn write_u16(&mut self, addr: usize, value: u16) {
        self.write(addr, (value & 0xFF) as u8);
        self.write(addr + 1, ((value >> 8) & 0xFF) as u8);
    }

    pub fn write(&mut self, addr: usize, value: u8) {
        self.tick(4);
        self.direct_write(addr, value)
    }

    pub fn direct_write(&mut self, addr: usize, value: u8) {
        match addr {
            0x0000..=0x3FFF => self.cartridge.write(addr, value),
            0x4000..=0x7FFF => self.cartridge.write(addr, value),
            0x8000..=0x9FFF => self.ppu.write(addr, value),
            0xA000..=0xBFFF => self.cartridge.write(addr, value),
            0xC000..=0xCFFF => self.ram[(addr - 0xC000)] = value,
            0xD000..=0xDFFF => self.ram[(addr - 0xC000)] = value,
            0xE000..=0xFDFF => self.ram[(addr - 0xE000)] = value,
            0xFE00..=0xFE9F => self.ppu.write(addr, value),
            0xFEA0..=0xFEFF => {}

            // Sound registers
            0xFF10..=0xFF3F => self.apu.write_reg(addr, value),

            P1_REG => self.buttons.write_p1(value),
            SB_REG => self.serial.write_reg(SB_REG, value),
            SC_REG => self.serial.write_reg(SC_REG, value),
            DIV_REG => self.timer.write_div(value),
            TIMA_REG => self.timer.tima = value,
            TMA_REG => self.timer.tma = value,
            TAC_REG => self.timer.tac = value,
            0xFF08 => println!("write to 0xFF08 - undocumented!: {}", value),
            IF_REG => self.set_if_reg(value),

            LCDC_REG => self.ppu.write(addr, value),
            STAT_REG => self.ppu.write(addr, value),
            SCY_REG => self.ppu.write(addr, value),
            SCX_REG => self.ppu.write(addr, value),
            LY_REG => self.ppu.write(addr, value),
            LYC_REG => self.ppu.write(addr, value),
            DMA_REG => self.dma.start(value),
            BGP_REG => self.ppu.write(addr, value),
            OBP0_REG => self.ppu.write(addr, value),
            OBP1_REG => self.ppu.write(addr, value),
            WY_REG => self.ppu.write(addr, value),
            WX_REG => self.ppu.write(addr, value),

            0xFF4D => println!("write to 0xFF4D - KEY1 (CGB only): {}", value),

            // 0xFF50: write 1 to disable bootstrap ROM
            0xFF50 => self.bootstrap_mode = false,

            // Invalid registers, that are still used by for example Tetris
            // https://www.reddit.com/r/EmuDev/comments/5nixai/gb_tetris_writing_to_unused_memory/
            0xFF7F => {}

            0xFF00..=0xFF7F => self.io_reg[(addr - 0xFF00) as usize] = value,
            0xFF80..=0xFFFE => self.internal_ram[(addr - 0xFF80) as usize] = value,

            IE_REG => {
                println!("SET IE TO {}", value);
                self.ie_reg = value
            }

            _ => panic!("Write to unhandled address: 0x{:x}", addr),
        };
    }

    #[allow(dead_code)]
    pub fn pop(&mut self) -> u16 {
        let sp = self.reg.sp;
        let v = self.read_u16(sp as usize);
        self.reg.sp = sp.wrapping_add(2);
        v
    }
}
