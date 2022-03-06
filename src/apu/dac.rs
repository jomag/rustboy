// Every channel has a DAC: a 4-bit digital-to-analog converter
// that generates a voltage from -1 to +1 for values 0 to 15.
pub struct DAC {
    pub powered_on: bool,
}

impl DAC {
    pub fn new() -> Self {
        DAC { powered_on: false }
    }

    pub fn convert(&self, inp: u8) -> f32 {
        assert!(inp & 0xF0 == 0);
        match self.powered_on {
            true => inp as f32 * (2.0 / 15.0) - 1.0,
            false => 0.0,
        }
    }
}
