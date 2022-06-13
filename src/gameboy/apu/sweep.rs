pub struct Sweep {
    duration: u8,
    decrement: bool,
    shift: u8,
    enabled: bool,
    counter: u8,
    shadow_frequency: u16,

    // To handle obscure behavior. See write_reg_nr10.
    has_calculated_in_decrement_mode: bool,
}

impl Sweep {
    pub fn new() -> Self {
        Sweep {
            // NR10, bit 6..4
            duration: 0,

            // NR10, bit 3
            decrement: false,

            // NR10, bit 3..0
            shift: 0,

            enabled: false,
            shadow_frequency: 0,
            counter: 0,

            has_calculated_in_decrement_mode: false,
        }
    }

    pub fn power_off(&mut self) {
        self.duration = 0;
        self.decrement = false;
        self.shift = 0;
        self.counter = 0;
    }

    pub fn read_reg_nr10(&self) -> u8 {
        let v = (self.duration << 4) | self.shift;
        if self.decrement {
            v | 0b1000
        } else {
            v
        }
    }

    pub fn write_reg_nr10(&mut self, value: u8, channel_enabled: &mut bool) {
        self.duration = (value >> 4) & 0b111;
        self.shift = value & 0b111;

        let prev = self.decrement;
        self.decrement = (value & 0b1000) != 0;
        if prev && !self.decrement && self.has_calculated_in_decrement_mode {
            // Obscure behavior: if the decrement bit is cleared after at least
            // one frequency calculation has been made in decrement mode
            // since the last trigger, the channel is immediately disabled.
            *channel_enabled = false;
        }
    }

    fn load_counter(&mut self) {
        self.counter = match self.duration {
            0 => 8,
            n => n,
        }
    }

    pub fn trigger(&mut self, channel_enabled: &mut bool, frequency: &mut u16) {
        self.shadow_frequency = *frequency;
        self.load_counter();
        self.enabled = self.duration != 0 || self.shift != 0;
        self.has_calculated_in_decrement_mode = false;

        if self.shift != 0 {
            // Overflow check
            self.calc_frequency(channel_enabled);
        }
    }

    fn calc_frequency(&mut self, channel_enabled: &mut bool) -> u16 {
        let mut f = self.shadow_frequency >> self.shift;

        if self.decrement {
            f = self.shadow_frequency - f;
            self.has_calculated_in_decrement_mode = true;
        } else {
            f = self.shadow_frequency + f;
        }

        if f > 2047 {
            *channel_enabled = false;
        }

        return f;
    }

    pub fn tick_128hz(&mut self, channel_enabled: &mut bool, frequency: &mut u16) {
        if self.counter > 0 {
            self.counter -= 1;

            if self.counter == 0 {
                self.counter = self.duration;
                if self.counter == 0 {
                    self.counter = 8;
                }

                if self.enabled && self.duration > 0 {
                    let new_frequency = self.calc_frequency(channel_enabled);

                    if new_frequency < 2048 && self.shift > 0 {
                        *frequency = new_frequency;
                        self.shadow_frequency = new_frequency;

                        self.calc_frequency(channel_enabled);
                    }
                }
            }
        }
    }
}
