
use registers::Registers;
use mmu::MMU;
use interrupt::handle_interrupts;

pub struct Cpu {
    pub reg: Registers,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
        reg: Registers::new(),
        }
    }
}
