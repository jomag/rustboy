use ringbuf::Producer;

use super::mmu::{SB_REG, SC_REG};

// This is a much simplified implementation of the serial transfer
// functionality in Gameboy. It only supports writing to a ringbuf,
// which can be used for easy monitoring of test roms etc.
//
// Full functionality includes giving interrupt when transfer
// finishes and shifting out bytes bit by bit, will simultaneously
// shifting in bytes from the other endpoint.

pub struct Serial {
    // SB (0xFF01): Serial Transfer Data
    reg_sb: u8,

    // SC (0xFF02): Serial Transfer Control
    // Bit 7: transfer start flag
    //        0 = no transfer in progress or requested
    //        1 = transfer in progress or requested
    // Bit 1: clock speed (cgb only)
    // Bit 0: shift clock (0 = external, 1 = internal)
    reg_sc: u8,

    pub output: Option<Producer<u8>>,
}

impl Serial {
    pub fn new(output: Option<Producer<u8>>) -> Self {
        Serial {
            reg_sb: 0,
            reg_sc: 0,
            output,
        }
    }

    pub fn read_reg(&self, address: usize) -> u8 {
        match address {
            SB_REG => self.reg_sb,
            SC_REG => self.reg_sc,
            _ => panic!(),
        }
    }

    pub fn write_reg(&mut self, address: usize, value: u8) {
        match address {
            SB_REG => self.reg_sb = value,
            SC_REG => {
                self.reg_sc = value;
                self.send(self.reg_sb);
            }
            _ => panic!(),
        }
    }

    fn send(&mut self, value: u8) {
        // Pushes SB register to output buffer, or prints
        // to stdout if no output buffer available.
        match self.output {
            Some(ref mut output) => output.push(value).expect("Serial output buffer full"),
            None => println!("{:x}: {}\n", value, value as char),
        }
    }
}
