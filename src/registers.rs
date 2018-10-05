
pub const Z_BIT:  u8 = 1 << 7;   // zero flag
pub const N_BIT:  u8 = 1 << 6;   // subtract flag
pub const H_BIT:  u8 = 1 << 5;   // half carry flag
pub const C_BIT:  u8 = 1 << 4;   // carry flag

pub struct Registers {
    // Registers
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,

    // Flags (the F register)
    pub zero: bool,
    pub neg: bool,
    pub half_carry: bool,
    pub carry: bool,

    // Inner state
    pub ime: u8, // 0 = disabled, 1 = enable after next op, 2 = enabled
    pub stopped: bool
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0, b: 0, c: 0, d: 0,
            e: 0, h: 0, l: 0,
            sp: 0, pc: 0,

            zero: false, neg: false,
            half_carry: false, carry: false,

            ime: 0,
            stopped: false
        }
    }

    pub fn af(&self) -> u16 {
        // Return 16-bit value of registers A and F
        let mut f: u8 = 0;
        if self.zero { f |= Z_BIT };
        if self.neg { f |= N_BIT };
        if self.half_carry { f |= H_BIT };
        if self.carry { f |= C_BIT };
        return (self.a as u16) << 8 | f as u16;
    }

    pub fn get_f(&self) -> u8 {
        let mut f: u8 = 0;
        if self.zero { f |= Z_BIT };
        if self.neg { f |= N_BIT };
        if self.half_carry { f |= H_BIT };
        if self.carry { f |= C_BIT };
        f
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
        // Note that this one is special:
        // The lower 4 bits of register F are not usable.
        // They should always remain zero.
        // Note that it's still possible to write to
        // self.f directly, so we should probably
        // not allow that either.
        self.a = ((value >> 8) & 0xFF) as u8;
        self.zero = value & (Z_BIT as u16) != 0;
        self.neg = value & (N_BIT as u16) != 0;
        self.half_carry = value & (H_BIT as u16) != 0;
        self.carry = value & (C_BIT as u16) != 0;
    }

    pub fn set_znhc(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.zero = z;
        self.neg = n;
        self.half_carry = h;
        self.carry = c;
    }
}
