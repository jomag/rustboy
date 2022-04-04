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
            true => ((inp as i32) * 0x1111 - 0x8000) as i16,
            false => 0,
        }
    }
}
