// Every channel has a DAC: a 4-bit digital-to-analog converter
// that generates a voltage from -1 to +1 for values 0 to 15.
pub struct DAC {
    pub powered_on: bool,
}

impl DAC {
    pub fn new() -> Self {
        DAC { powered_on: false }
    }

    pub fn convert(&self, inp: u8) -> i16 {
        assert!(inp & 0xF0 == 0);
        match self.powered_on {
            // true => inp as f32 * (2.0 / 15.0) - 1.0
            // true => -0x7800 + (inp as i16) * 0x1000,
            true => (inp as i16) * 0x100 - 0x780,
            false => 0,
        }
    }
}
