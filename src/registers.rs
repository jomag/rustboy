
pub const Z_BIT:  u8 = 1 << 7;   // zero flag
pub const N_BIT:  u8 = 1 << 6;   // subtract flag
pub const H_BIT:  u8 = 1 << 5;   // half carry flag
pub const C_BIT:  u8 = 1 << 4;   // carry flag

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub ime: bool,
    pub stopped: bool
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0, b: 0, c: 0, d: 0,
            e: 0, f: 0, h: 0, l: 0,
            sp: 0, pc: 0,
            ime: false,
            stopped: false
        }
    }

    pub fn c_flag(&self) -> bool {
        (self.f & C_BIT) != 0
    }

    pub fn z_flag(&self) -> bool {
        (self.f & Z_BIT) != 0
    }

    pub fn set_z_flag(&mut self, val: bool) {
        if val {
            self.f |= Z_BIT;
        } else {
           self.f &= !Z_BIT;
        }
    }

    pub fn set_n_flag(&mut self) {
        self.f |= N_BIT;
    }

    pub fn clear_n_flag(&mut self) {
        self.f &= !N_BIT;
    }

    pub fn af(&self) -> u16 {
        // Return 16-bit value of registers A and F
        return (self.a as u16) << 8 | self.f as u16;
    }

    pub fn bc(&self) -> u16 {
        // Return 16-bit value of registers B and C
        return (self.b as u16) << 8 | self.c as u16;
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value >> 8) & 0xFF) as u8;
        self.c = (value & 0xFF) as u8;
    }

    pub fn de(&self) -> u16 {
        // Return 16-bit value of registers D and E
        return (self.d as u16) << 8 | self.e as u16;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = ((value >> 8) & 0xFF) as u8;
        self.e = (value & 0xFF) as u8;
    }

    pub fn hl(&self) -> u16 {
        // Return 16-bit value of registers H and L
        return (self.h as u16) << 8 | self.l as u16;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value >> 8) & 0xFF) as u8;
        self.l = (value & 0xFF) as u8;
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = ((value >> 8) & 0xFF) as u8;
        self.f = (value & 0xFF) as u8;
    }

    pub fn set_carry(&mut self, en: bool) {
        if en {
            self.f |= C_BIT;
        } else {
            self.f &= !C_BIT;
        }
    }
}
