use crate::mmu::MMU;

pub struct Emu {
    pub mmu: MMU,
}

impl Emu {
    pub fn new() -> Self {
        Emu { mmu: MMU::new() }
    }

    pub fn init(&mut self) {
        self.mmu.init();
    }

    pub fn load_bootstrap(&mut self, path: &str) -> usize {
        self.mmu.load_bootstrap(&path)
    }

    pub fn load_cartridge(&mut self, path: &str) {
        println!("enter load cartridge");
        let x = self.mmu.load_cartridge(path);
        println!("lkeave load cartridge");
        x
    }
}
