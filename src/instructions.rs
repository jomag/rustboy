
use registers::{ Registers, Z_BIT, N_BIT, H_BIT, C_BIT };

pub fn xor_op(reg: &mut Registers, a: u8, value: u8) -> u8 {
    // Flags: Z 0 0 0
    let res = a ^ value;
    reg.f &= !(Z_BIT | N_BIT | H_BIT | C_BIT);
    if res == 0 {
        reg.f |= Z_BIT;
    }
    res
}

pub fn bit_op(reg: &mut Registers, bit: u8, value: u8) {
    // Test if bit in register is set
    // Flags: Z 0 1 -
    if value & (1 << bit) == 0 {
        reg.f &= !N_BIT;
        reg.f |= Z_BIT | H_BIT;
    } else {
        reg.f &= !(Z_BIT | N_BIT);
        reg.f |= H_BIT;
    }
}

pub fn inc_op(reg: &mut Registers, value: u8) -> u8 {
    // Flags: Z 0 H -
    let new_value = if value == 255 { 0 } else { value + 1 };

    if new_value == 0 {
        reg.f |= Z_BIT;
    } else {
        reg.f &= !Z_BIT;
    }

    reg.f &= !N_BIT;

    if value < 255 && ((value & 0xF) + 1) & 0x10 != 0 {
        reg.f |= H_BIT;
    } else {
        reg.f &= !H_BIT;
    }

    new_value
}

pub fn inc16_op(value: u16) -> u16 {
    return if value == 0xFFFF { 0 } else { value + 1 };
}

pub fn dec_op(reg: &mut Registers, value: u8) -> u8 {
    // Flags: Z 1 H -
    let new_value = if value == 0 { 255 } else { value - 1 };

    if new_value == 0 {
        reg.f |= Z_BIT | N_BIT;
    } else {
        reg.f &= !Z_BIT;
        reg.f |= N_BIT;
    }

    // FIXME: handle half-carry flag
    new_value
}

pub fn rl_op(reg: &mut Registers, value: u8) -> u8 {
    // Rotate left with carry flag
    // Flags: Z 0 0 C
    let mut t = (value as u32) << 1;

    if reg.f & C_BIT != 0 {
        t |= 1;
    }

    if t & 256 == 0 {
        reg.f &= !C_BIT;
    } else {
        reg.f |= C_BIT;
    }

    if t == 0 {
        reg.f |= Z_BIT;
    } else {
        reg.f &= Z_BIT;
    }

    reg.f &= !(N_BIT | H_BIT);

    (t & 0xFF) as u8
}
