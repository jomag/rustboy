
// References:
// http://gbdev.gg8.se/wiki/articles/Timer_and_Divider_Registers
// http://gbdev.gg8.se/wiki/articles/Timer_Obscure_Behaviour

use mmu::{ MMU, TIMA_REG, IF_REG };

const CLOCK_SELECTION: [u16; 4] = [ 4096, 64, 256, 1024 ];

const TAC_ENABLE_BIT: u8 = 4;

pub struct Timer {
    // The internal 16-bit counter. DIV is the top 8 bits.
    pub cycle: u16,

    // TIMA is incremented when one specific bit goes
    // from high to low. Therefore we need to store
    // the previous cycle to compare with, because
    // the bit might have gone low because DIV has
    // been written to, and TIMA should be incremented
    // in that case as well.
    pub prev_cycle: u16,

    // TAC register: controller register
    // Bit 2: 0 = stop timer, 1 = start timer
    // Bit 1-0: Clock select
    //
    // Clock selection:
    // 00: 4096 Hz
    // 01: 262 144 Hz
    // 10: 65 536 Hz
    // 11: 16 384 Hz
    pub tac: u8,

    // TIMA register: timer counter
    // When TIMA overflows an interrupt is generated and
    // TIMA is reset to the value of TMA
    pub tima: u8,

    // TMA register: reset value of TIMA
    pub tma: u8,

    interrupt: bool
}

impl Timer {
            pub fn new() -> Self {
        Timer {
            cycle: 0,
            prev_cycle: 0,
            tac: 0,
            tima: 0,
            tma: 0,
            interrupt: false
        }
    }

    pub fn write_div(&mut self, value: u8) {
        // Value is ignored: no matter what value is written
        // the cycle counter is always reset to zero
        self.cycle = 0;
    }

    pub fn read_div(&self) -> u8 {
        (self.cycle >> 8) as u8
    }

    pub fn update(&mut self, cycles: u32) {
        for _ in 0..cycles {
            self.one_cycle();
        }
    }

    fn one_cycle(&mut self) {
        self.prev_cycle = self.cycle;
        self.cycle = self.cycle.wrapping_add(1);

        if self.tac & TAC_ENABLE_BIT != 0 {
            let bit = CLOCK_SELECTION[(self.tac & 3) as usize];
            if (self.prev_cycle & bit) != 0 && (self.cycle & bit) == 0 {
                if self.tima == 0xFF {
                    //println!("TIMER INTERRUPT!");
                    self.interrupt = true;
                    self.tima = self.tma;
                } else {
                    self.tima = self.tima + 1;
                }
            }
        }
    }
}
