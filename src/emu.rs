use mmu::MMU;

pub struct Emu {
    pub mmu: MMU,
}

impl Emu {
    pub fn new(sample_rate: u32) -> Self {
        Emu {
            mmu: MMU::new(sample_rate),
        }
    }

    pub fn init(&mut self) {
        self.mmu.init();
    }

    pub fn load_bootstrap(&mut self, path: &str) -> usize {
        self.mmu.load_bootstrap(&path)
    }

    pub fn load_cartridge(&mut self, path: &str) {
        self.mmu.load_cartridge(&path)
    }
}
