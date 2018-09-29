
// References:
// http://gbdev.gg8.se/wiki/articles/Timer_and_Divider_Registers
// http://gbdev.gg8.se/wiki/articles/Timer_Obscure_Behaviour

use memory::{ Memory, TIMA_REG, IF_REG };
use interrupt::TMR_BIT;

const CLOCK_SELECTION: [u16; 4] = [ 1023, 15, 63, 255 ];

const TAC_ENABLE_BIT: u8 = 4;

pub struct Timer {
    // The internal 16-bit counter. DIV is the top 8 bits.
    cycle: u16,

    // TAC register: controller register
    // Bit 2: 0 = stop timer, 1 = start timer
    // Bit 1-0: Clock select
    //
    // Clock selection:
    // 00: 4096 Hz
    // 01: 262 144 Hz
    // 10: 65 536 Hz
    // 11: 16 384 Hz
    tac: u8,

    // TIMA register: timer counter
    // When TIMA overflows an interrupt is generated and
    // TIMA is reset to the value of TMA
    tima: u8,

    // TMA register: reset value of TIMA
    tma: u8

    interrupt: u8
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            cycle: 0,
            tac: 0,
            tima: 0,
            tma: 0,
            interrupt: 0
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
        self.cycle.wrapping_add(1);

        if self.tac & TAC_ENABLE_BIT != 0 {
            // Timer enabled
            if self.cycle & CLOCK_SELECTION[(self.tac & 3) as usize] == 0 {
                if self.tima == 0xFF {
                    //println!("TIMER INTERRUPT!");
                    self.interrupt = true;
                    self.tima = 0
                } else {
                    self.tima = self.tima + 1;
                }
            }
        }
    }
}
