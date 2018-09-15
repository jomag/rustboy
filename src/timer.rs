
use memory::{ TAC_REG, TIMA_REG, TMA_REG, IF_REG, Memory };
use interrupt::TMR_BIT;

const clock_selection: [u32; 4] = [ 4095, 262143, 65535, 16383 ];

pub struct Timer {
    cycle: u32
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            cycle: 0
        }
    }

    pub fn update(&mut self, mem: &mut Memory, cycles: u32) {
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

        if tac & 4 == 0 {
            self.cycle += cycles
        } else {
            for _ in 0..(cycles / 4) {
                if self.cycle & clock_selection[(tac & 3) as usize] == 0 {
                    let mut tima: u32 = (mem.mem[TIMA_REG as usize] as u32) + 4;
                    if tima > 0xFF {
                        // On overflow, set TIMA to the value of the
                        // timer modulo (TMA).
                        // FIXME: if TMA has a very high value
                        //        it could cause TIMA to immediately
                        //        overflow again! This is only because
                        //        we add 4 cycles at once.
                        tima = tima & 0xFF + (mem.mem[TMA_REG as usize] as u32);
                        mem.mem[IF_REG as usize] |= TMR_BIT;
                    }
                    mem.mem[TIMA_REG as usize] = tima as u8;
                }
            }
        }
    }
}
