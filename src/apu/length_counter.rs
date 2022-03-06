use crate::emu::Machine;

// All channels have a length counter which counts down and disables
// the channel when it reaches zero. The length counter can be
// disabled.
pub struct LengthCounter {
    // Max length value. 256 for ch 3 (wave), 64 for the others
    max: u16,

    machine: Machine,
    _enabled: bool,
    pub value: u16,
}

impl LengthCounter {
    pub fn new(machine: Machine, max: u16) -> Self {
        LengthCounter {
            machine,
            max,
            _enabled: false,
            value: 0,
        }
    }

    pub fn power_off(&mut self) {
        self._enabled = false;
        self.value = match self.machine {
            Machine::GameBoyDMG => self.value,
            Machine::GameBoyCGB => 0,
            _ => panic!("unsupported machine type"),
        }
    }

    pub fn write_reg_nrx1(&mut self, value: u8) {
        let mask = self.max - 1;
        self.value = self.max - (value as u16 & mask);
    }

    pub fn next_seq_step_will_not_count_down(seq_step: u8) -> bool {
        return seq_step % 2 == 0;
    }

    pub fn is_enabled(&self) -> bool {
        self._enabled
    }

    // Enable the counter. If the next sequencer step will not clock
    // the length counter, the counter value is immediately decremented
    // which may cause the channel to become disabled.
    //
    // If this function returns true, the channel should be disabled.
    pub fn enable(&mut self, en: bool, seq_step: u8) -> bool {
        if en {
            if !self._enabled {
                self._enabled = true;
                if LengthCounter::next_seq_step_will_not_count_down(seq_step) && self.value > 0 {
                    return self.count_down();
                }
            }
        } else {
            self._enabled = false;
        }

        false
    }

    // When triggered, if length counter is 0 it should
    // be reset to 64 (256 for wave channel).
    //
    // Obscure behavior:
    // If channel is triggered when next sequencer step will not
    // clock the length counter, the length counter is immediately
    // decremented.
    pub fn trigger(&mut self, reset_value: u16, seq_step: u8) {
        if self.value == 0 {
            self.value = reset_value;

            if self._enabled && LengthCounter::next_seq_step_will_not_count_down(seq_step) {
                self.value -= 1;
            }
        }
    }

    // This function should be called by the 256 Hz frame sequencer
    // tick. If it returns true, it has reached zero and the channel
    // should be disabled.
    pub fn count_down(&mut self) -> bool {
        if self._enabled {
            match self.value {
                0 => return false,
                1 => {
                    self.value = 0;
                    return true;
                }
                _ => {
                    self.value -= 1;
                    return false;
                }
            }
        } else {
            return false;
        }
    }
}
