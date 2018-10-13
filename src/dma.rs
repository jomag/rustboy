
use mmu::{ MMU, OAM_OFFSET };

pub struct DMA {
    pub start_address: u16,
    pub step: u16,
    pub state: u8,
    oam: [u8; 0xA0]
}

impl DMA {
    pub fn new() -> Self {
        DMA {
            start_address: 0,
            step: 0,
            state: 0,
            oam: [0; 0xA0]
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        if self.state == 2 {
            return 0xFF
        } else {
            return self.oam[address as usize];
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.oam[address as usize] = value;
    }

    pub fn start(&mut self, start: u8) {
        self.start_address = (start as u16) << 8;
        self.step = 0;

        if self.state == 0 {
            println!("START: {}", start);
            self.state = 1;
        } else {
            println!("RESTART: {}", start);
            self.state = 3;
        }
    }

    pub fn update(&mut self) {
        match self.state {
            0 => {}
            1 => { self.state = 2 }
            3 => { self.state = 2 }
            2 => {
                if self.step == 0xFF {
                    self.state = 0;
                } else {
                    self.step += 1;
                }
            }
            _ => { panic!("invalid dma state: {}", self.state) }
        }
    }
}
