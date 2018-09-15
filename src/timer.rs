
use memory::{ TAC_REG, TIMA_REG };
use memory::Memory;

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
                    if mem.mem[TIMA_REG as usize] == 252 {
                        mem.mem[TIMA_REG as usize] = 0;
                    } else {
                        mem.mem[TIMA_REG as usize] += 4;
                    }
                    println!("TIMA new value: {}", mem.mem[TIMA_REG as usize]);
                }
            }
        }
    }
}
