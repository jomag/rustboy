pub mod apu;
pub mod buttons;
pub mod cartridge;
mod dma;
pub mod emu;
pub mod instructions;
mod interrupt;
pub mod mmu;
pub mod ppu;
pub mod registers;
mod serial;
mod timer;

pub const CLOCK_SPEED: usize = 4194304;
pub const CYCLES_PER_FRAME: usize = 70224;
pub const BOOTSTRAP_ROM: &str = "rom/boot.gb";
pub const CARTRIDGE_ROM: &str = "rom/tetris.gb";
