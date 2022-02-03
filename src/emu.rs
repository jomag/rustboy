use crate::mmu::MMU;

#[derive(Copy, Clone)]
pub enum Machine {
    GameBoyDMG,

    // FIXME: is this the correct abbreviation?
    GameBoyCGB,
}

pub struct Emu {
    pub mmu: MMU,
    pub machine: Machine,
}

impl Emu {
    pub fn new(machine: Machine) -> Self {
        Emu {
            mmu: MMU::new(machine),
            machine,
        }
    }

    pub fn reset(&mut self) {
        self.mmu.reset();
    }

    pub fn init(&mut self) {
        self.mmu.init();
    }

    pub fn load_bootstrap(&mut self, path: &str) -> usize {
        self.mmu.load_bootstrap(&path)
    }

    pub fn load_cartridge(&mut self, path: &str) {
        self.mmu.load_cartridge(path);
    }
}
