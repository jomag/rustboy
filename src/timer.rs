
// References:
// http://gbdev.gg8.se/wiki/articles/Timer_and_Divider_Registers
// http://gbdev.gg8.se/wiki/articles/Timer_Obscure_Behaviour

use memory::{ DIV_REG, TAC_REG, TIMA_REG, TMA_REG, IF_REG };
use interrupt::TMR_BIT;

const CLOCK_SELECTION: [u16; 4] = [ 1023, 15, 63, 255 ];

pub struct Timer {
    // The internal 16-bit counter. DIV is the top 8 bits.
    cycle: u16
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            cycle: 0
        }
    }

    pub fn write_div(&mut self, value: u8) {
        // Value is ignored: no matter what value is written
        // the cycle counter is always reset to zero
        self.cycle = 0;
    }

    pub fn read_div(&self) -> u8 {
        self.cycle
    }

    pub fn update(&mut self, mem: &mut Memory, cycles: u32) {
        for _ in 0..cycles {
            self.one_cycle(mem);
        }
    }

    fn one_cycle(&mut self, mem: &mut Memory) {
        self.cycle.wrapping_add(1);

        // Update DIV register. DIV is the upper 8 bits
        // of a 16 bit counter that increments on each
        // clock cycle.
        mem.mem[DIV_REG as usize] = (self.cycle >> 8) as u8;

        // TAC register:
        // Bit 2: 0 = stop timer, 1 = start timer
        // Bit 1-0: Clock select
        //
        // Clock selection:
        // 00: 4096 Hz
        // 01: 262 144 Hz
        // 10: 65 536 Hz
        // 11: 16 384 Hz
        let tac = mem.mem[TAC_REG as usize];

        if tac & 4 != 0 {
            // Timer enabled
            if self.cycle & CLOCK_SELECTION[(tac & 3) as usize] == 0 {
                let mut tima = mem.mem[TIMA_REG as usize];
                if tima == 0xFF {
                    //println!("TIMER INTERRUPT!");
                    mem.mem[IF_REG as usize] |= TMR_BIT;
                    tima = 0
                } else {
                    tima = tima + 1;
                }
                mem.mem[TIMA_REG as usize] = tima;
            }
        }
    }
}
