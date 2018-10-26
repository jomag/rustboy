
use registers::Registers;

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
