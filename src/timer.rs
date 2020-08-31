// References:
// http://gbdev.gg8.se/wiki/articles/Timer_and_Divider_Registers
// http://gbdev.gg8.se/wiki/articles/Timer_Obscure_Behaviour

use interrupt::IF_TMR_BIT;

const CLOCK_SELECTION: [u16; 4] = [512, 8, 32, 128];

const TAC_ENABLE_BIT: u8 = 4;

pub struct Timer {
    // Absolut cycle count since start of emulator.
    // This is only used for statistics and debugging.
    // It is not used by the emulator.
    pub abs_cycle: u64,

    // The internal 16-bit counter. DIV is the top 8 bits.
    pub cycle: u16,

    // TIMA is incremented when one specific bit goes
    // from high to low. Therefore we need to store
    // the previous cycle to compare with, because
    // the bit might have gone low because DIV has
    // been written to, and TIMA should be incremented
    // in that case as well.
    pub prev_bit_state: bool,
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

    pub irq: u8,

    pub trigger_debug: bool,

    // Break at absolute cycle. Cycle 0 is ignored.
    pub abs_cycle_breakpoint: u64,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            abs_cycle: 0,
            cycle: 0,
            prev_cycle: 0,
            prev_bit_state: false,
            tac: 0,
            tima: 0,
            tma: 0,
            irq: 0,
            trigger_debug: false,
            abs_cycle_breakpoint: 0,
        }
    }

    pub fn write_div(&mut self, _value: u8) {
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

    fn x_one_cycle(&mut self) {
        self.cycle = self.cycle.wrapping_add(1);

        if self.tac & TAC_ENABLE_BIT != 0 {
            let bit = CLOCK_SELECTION[(self.tac & 3) as usize];
            if (self.prev_cycle & bit) != 0 && (self.cycle & bit) == 0 {
                if self.tima == 0xFF {
                    self.irq |= IF_TMR_BIT;
                    self.tima = self.tma;
                } else {
                    self.tima = self.tima + 1;
                }
            }
        }

        self.prev_cycle = self.cycle;
    }

    fn one_cycle(&mut self) {
        self.abs_cycle = self.abs_cycle.wrapping_add(1);

        if self.abs_cycle == self.abs_cycle_breakpoint {
            self.trigger_debug = true;
        }

        self.cycle = self.cycle.wrapping_add(1);

        let bit = if self.tac & TAC_ENABLE_BIT != 0 {
            CLOCK_SELECTION[(self.tac & 3) as usize]
        } else {
            0
        };

        let bit_state = self.cycle & bit != 0;

        if self.prev_bit_state && !bit_state {
            if self.tima == 0xFF {
                self.irq |= IF_TMR_BIT;
                self.tima = self.tma;
            } else {
                self.tima = self.tima + 1;
            }
        }

        self.prev_bit_state = bit_state;
    }
}
