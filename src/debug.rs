
use instructions::op_length;
use memory::Memory;
use registers::{ Registers, Z_BIT, H_BIT, N_BIT, C_BIT };

fn add_i8_to_u16(a: u16, b: i8) -> u16 {
    if b > 0 {
        return a + b as u16;
    } else {
        return a - (-b) as u16;
    }
}

pub fn print_registers(reg: &Registers) {
    print!("  A: 0x{:02X} B: 0x{:02X} C: 0x{:02X} D: 0x{:02X} ", reg.a, reg.b, reg.c, reg.d);
    println!("E: 0x{:02X} F: 0x{:02X} H: 0x{:02X} L: 0x{:02X}", reg.e, reg.f, reg.h, reg.l);
    println!("  SP: 0x{:04X} PC: 0x{:04X}", reg.sp, reg.pc);
    println!(
        "  Flags: Z={}, N={}, H={}, C={}",
        if (reg.f & Z_BIT) == 0 { 0 } else { 1 },
        if (reg.f & N_BIT) == 0 { 0 } else { 1 },
        if (reg.f & H_BIT) == 0 { 0 } else { 1 },
        if (reg.f & C_BIT) == 0 { 0 } else { 1 },
    )
}

pub fn format_mnemonic(mem: &Memory, addr: u16) -> String {
    let op: u8 = mem.read(addr);
    match op {
        0x00 => { "NOP".to_string() }
        0x01 => { format!("LD  BC, ${:04X}", mem.read_u16(addr + 1)) }

        // INC n: increment register n
        0x04 => { "INC  B".to_string() }
        0x0C => { "INC  C".to_string() }
        0x14 => { "INC  D".to_string() }
        0x1C => { "INC  E".to_string() }
        0x24 => { "INC  H".to_string() }
        0x2C => { "INC  L".to_string() }
        0x3C => { "INC  A".to_string() }

        // INC nn: increment 16-bit register nn
        0x13 => { "INC  DE".to_string() }
        0x23 => { "INC  HL".to_string() }

        // DEC n: decrement register n
        0x05 => { "DEC  B".to_string() }
        0x0D => { "DEC  C".to_string() }
        0x15 => { "DEC  D".to_string() }
        0x1D => { "DEC  E".to_string() }
        0x25 => { "DEC  H".to_string() }
        0x2D => { "DEC  L".to_string() }
        0x3D => { "DEC  A".to_string() }

        // LD n, d: load immediate into register n
        0x06 => { format!("LD   B, ${:02X}", mem.read(addr + 1)) }
        0x0E => { format!("LD   C, ${:02X}", mem.read(addr + 1)) }
        0x16 => { format!("LD   D, ${:02X}", mem.read(addr + 1)) }
        0x1E => { format!("LD   E, ${:02X}", mem.read(addr + 1)) }
        0x26 => { format!("LD   H, ${:02X}", mem.read(addr + 1)) }
        0x2E => { format!("LD   L, ${:02X}", mem.read(addr + 1)) }
        0x3E => { format!("LD   A, ${:02X}", mem.read(addr + 1)) }

        0x11 => {
            let lo = mem.read(addr + 1);
            let hi = mem.read(addr + 2);
            format!("LD   DE, ${:02X}{:02X}", hi, lo)
        }
        0x17 => { "RLA".to_string() }
        0x18 => {
            let rel = mem.read_i8(addr + 1);
            let abs = add_i8_to_u16(addr + 2, rel);
            format!("JR   {}  ; jump to 0x{:04X}", rel, abs)
        }
        0x1A => { "LD   A, (DE)".to_string() }

        0x20 => {
            let rel = mem.read_i8(addr + 1);
            let abs = add_i8_to_u16(addr + 2, rel);
            format!("JR   NZ, {}    ; jump to 0x{:04X}", rel, abs)
        }
        0x21 => {
            let lo = mem.read(addr + 1);
            let hi = mem.read(addr + 2);
            format!("LD   HL, ${:02X}{:02X}", hi, lo)
        }
        0x22 => { "LD   (HL+), A".to_string() }
        0x28 => {
            let rel = mem.read_i8(addr + 1);
            let abs = add_i8_to_u16(addr + 2, rel);
            format!("JR   Z, {}        ; jump to 0x{:04X}", rel, abs)
        }

        0x31 => {
            let lo = mem.read(addr + 1);
            let hi = mem.read(addr + 2);
            format!("LD   SP, ${:02X}{:02X}", hi, lo)
        }
        0x32 => { "LDD  (HL), A".to_string() }

        0x40 => { "LD   B, B".to_string() }
        0x41 => { "LD   B, C".to_string() }
        0x42 => { "LD   B, D".to_string() }
        0x43 => { "LD   B, E".to_string() }
        0x44 => { "LD   B, H".to_string() }
        0x45 => { "LD   B, L".to_string() }
        0x46 => { "LD   B, (HL)".to_string() }
        0x47 => { "LD   B, A".to_string() }

        0x48 => { "LD   C, B".to_string() }
        0x49 => { "LD   C, C".to_string() }
        0x4A => { "LD   C, D".to_string() }
        0x4B => { "LD   C, E".to_string() }
        0x4C => { "LD   C, H".to_string() }
        0x4D => { "LD   C, L".to_string() }
        0x4E => { "LD   C, (HL)".to_string() }
        0x4F => { "LD   C, A".to_string() }

        0x50 => { "LD   D, B".to_string() }
        0x51 => { "LD   D, C".to_string() }
        0x52 => { "LD   D, D".to_string() }
        0x53 => { "LD   D, E".to_string() }
        0x54 => { "LD   D, H".to_string() }
        0x55 => { "LD   D, L".to_string() }
        0x56 => { "LD   D, (HL)".to_string() }
        0x57 => { "LD   D, A".to_string() }

        0x58 => { "LD   E, B".to_string() }
        0x59 => { "LD   E, C".to_string() }
        0x5A => { "LD   E, D".to_string() }
        0x5B => { "LD   E, E".to_string() }
        0x5C => { "LD   E, H".to_string() }
        0x5D => { "LD   E, L".to_string() }
        0x5E => { "LD   E, (HL)".to_string() }
        0x5F => { "LD   E, A".to_string() }

        0x60 => { "LD   H, B".to_string() }
        0x61 => { "LD   H, C".to_string() }
        0x62 => { "LD   H, D".to_string() }
        0x63 => { "LD   H, E".to_string() }
        0x64 => { "LD   H, H".to_string() }
        0x65 => { "LD   H, L".to_string() }
        0x66 => { "LD   H, (HL)".to_string() }
        0x67 => { "LD   H, A".to_string() }

        0x68 => { "LD   L, B".to_string() }
        0x69 => { "LD   L, C".to_string() }
        0x6A => { "LD   L, D".to_string() }
        0x6B => { "LD   L, E".to_string() }
        0x6C => { "LD   L, H".to_string() }
        0x6D => { "LD   L, L".to_string() }
        0x6E => { "LD   L, (HL)".to_string() }
        0x6F => { "LD   L, A".to_string() }

        0x77 => { "LD   (HL), A".to_string() }
        0x78 => { "LD   A, B".to_string() }
        0x7B => { "LD   A, E".to_string() }
        0x7C => { "LD   A, H".to_string() }
        0x7D => { "LD   A, L".to_string() }

        0x86 => { "ADD  A, (HL)".to_string() }

        0x90 => { "SUB  B".to_string() }

        0xA8 => { "XOR  B".to_string() }
        0xA9 => { "XOR  C".to_string() }
        0xAA => { "XOR  D".to_string() }
        0xAB => { "XOR  E".to_string() }
        0xAC => { "XOR  H".to_string() }
        0xAD => { "XOR  L".to_string() }
        0xAF => { "XOR  A".to_string() }

        0xBE => { "CP   (HL)".to_string() }

        0xC1 => { "POP  BC".to_string() }
        0xC3 => { format!("JP   0x{:04X}", mem.read_u16(addr + 1)) }
        0xC4 => { format!("CALL  NZ, ${:04X}", mem.read_u16(addr + 1)) }
        0xC5 => { "PUSH BC".to_string() }
        0xC9 => { "RET".to_string() }
        0xCB => {
            let op2 = mem.read(addr + 1);
            match op2 {
                0x11 => { "RL   C".to_string() }
                0x7C => { "BIT 7, h".to_string() }
                _ => {
                    panic!("invalid instruction op code: 0x{:02X}{:02X}", op, op2);
                }
            }
        }
        0xCD => { format!("CALL ${:04X}", mem.read_u16(addr + 1)) }

        0xE0 => { format!("LD   ($FF00+${:02X}), A", mem.read(addr + 1)) }
        0xE2 => { "LD   ($FF00+C), A".to_string() }
        0xEA => { format!("LD   (${:04X}), A", mem.read_u16(addr + 1)) }

        0xF0 => { format!("LD   A, ($FF00+${:02X})", mem.read(addr + 1)) }
        0xFE => { format!("CP   ${:02X}", mem.read(addr + 1)) }

        _ => {
            panic!("invalid instruction op code at 0x{:04X}: 0x{:02X}", addr, op);
        }
    }
}

pub fn print_listing(mem: &Memory, addr: u16, line_count: i32) -> u16 {
    let mut a = addr;
    for _n in 0..line_count {
        println!("0x{:04X}: {}", a, format_mnemonic(&mem, a));
        a = a + (op_length(mem.read(addr)) as u16);
    }
    a
}

pub fn address_type(addr: u16) -> String {
    if addr < 0x4000 {
        return "ROM bank #0".to_string();
    }

    if addr < 0x8000 {
        return "ROM bank #1 (switchable)".to_string();
    }

    if addr < 0xA000 {
        return "Video RAM".to_string();
    }

    if addr < 0xC000 {
        return "Switchable RAM bank".to_string();
    }

    if addr < 0xE000 {
        return "Internal RAM (1)".to_string();
    }

    if addr < 0xFE00 {
        return "Echo of internal RAM".to_string();
    }

    if addr < 0xFEA0 {
        return "Sprite Attrib Memory (OAM)".to_string();
    }

    if addr < 0xFF00 {
        return "Empty memory block, unusable for I/O (1)".to_string();
    }

    if addr < 0xFF4C {
        return "I/O ports".to_string();
    }

    if addr < 0xFF80 {
        return "Empty memory block, unusable for I/O (2)".to_string();
    }

    if addr < 0xFFFF {
        return "Internal RAM (2)".to_string();
    }

    return "Interrupt Enable Register".to_string();
}
