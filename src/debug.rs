use instructions::op_length;
use mmu::{IE_REG, IF_REG, MMU};
use std::fs::File;
use std::io::Write;
use timer::Timer;

fn add_i8_to_u16(a: u16, b: i8) -> u16 {
    if b > 0 {
        return a + b as u16;
    } else {
        return a - (-b) as u16;
    }
}

pub fn log_state(file: &mut File, mmu: &MMU) {
    let f = mmu.reg.get_f();
    file.write_fmt(format_args!(
        "A:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} F:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} Op: {:02x} {:02x} DIV: {:02x}\n",
        mmu.reg.a, mmu.reg.b, mmu.reg.c, mmu.reg.d,
        mmu.reg.e, f, mmu.reg.h, mmu.reg.l,
        mmu.reg.sp, mmu.reg.pc, mmu.direct_read(mmu.reg.pc), mmu.direct_read(mmu.reg.pc + 1),
        mmu.timer.read_div()
    ));
}

pub fn print_stack(mmu: &MMU, sp: u16) {
    let mut a: u16 = 0xFFFC;

    if sp < 0xFF80 {
        println!("  stack: SP at 0x{:04X}. Stack corrupted?", sp);
        return;
    }

    if sp > a {
        println!("  stack: empty");
    } else {
        print!("  stack:");
        while a >= sp {
            print!(" {:04X}", mmu.direct_read_u16(a));
            a -= 2;
        }
        println!();
    }
}

pub fn print_registers(mmu: &MMU) {
    print!(
        "  A: 0x{:02X} B: 0x{:02X} C: 0x{:02X} D: 0x{:02X} ",
        mmu.reg.a, mmu.reg.b, mmu.reg.c, mmu.reg.d
    );

    println!(
        "E: 0x{:02X} F: 0x{:02X} H: 0x{:02X} L: 0x{:02X}",
        mmu.reg.e,
        mmu.reg.get_f(),
        mmu.reg.h,
        mmu.reg.l
    );

    println!(
        "  SP: 0x{:04X} PC: 0x{:04X} Cycle: 0x{:04X}/{}",
        mmu.reg.sp, mmu.reg.pc, mmu.timer.cycle, mmu.timer.cycle
    );

    println!(
        "  Flags: Z={}, N={}, H={}, C={}",
        if mmu.reg.zero { 1 } else { 0 },
        if mmu.reg.neg { 1 } else { 0 },
        if mmu.reg.half_carry { 1 } else { 0 },
        if mmu.reg.carry { 1 } else { 0 }
    );

    print_interrupt_state(&mmu);
    print_timer_state(&mmu.timer);
    print_stack(&mmu, mmu.reg.sp);

    if mmu.reg.halted {
        println!("  CPU is halted");
    }

    if mmu.reg.stopped {
        println!("  CPU is stopped")
    }
}

pub fn print_interrupt_state(mmu: &MMU) {
    println!(
        "  IME: {} IE: 0x{:02X} IF: 0x{:02X}",
        if mmu.reg.ime == 0 { 0 } else { 1 },
        mmu.direct_read(IE_REG),
        mmu.direct_read(IF_REG)
    );
}

pub fn print_timer_state(timer: &Timer) {
    println!(
        "  TAC: 0x{:02X} TIMA: 0x{:02X} TMA: 0x{:02X} Abs cycle: {}",
        timer.tac, timer.tima, timer.tma, timer.abs_cycle
    );
}

const SIMPLE_MNEMONICS: [&str; 256] = [
    // 0x00
    "NOP",
    "",
    "LD   (BC), A",
    "INC  BC",
    "INC  B",
    "DEC  B",
    "",
    "RLCA",
    // 0x08
    "",
    "ADD  HL, BC",
    "LD   A, (BC)",
    "DEC  BC",
    "INC  C",
    "DEC  C",
    "",
    "RRCA",
    // 0x10
    "STOP 0",
    "",
    "LD   (DE), A",
    "INC  DE",
    "INC  D",
    "DEC  D",
    "",
    "RLA",
    // 0x18
    "",
    "ADD  HL, DE",
    "LD   A, (DE)",
    "DEC  DE",
    "INC  E",
    "DEC  E",
    "",
    "RRA",
    // 0x20
    "",
    "",
    "LD   (HL+), A",
    "INC  HL",
    "INC  H",
    "DEC  H",
    "",
    "DAA",
    // 0x28
    "",
    "ADD  HL, HL",
    "LD   A, (HL+)",
    "DEC  HL",
    "INC  L",
    "DEC  L",
    "",
    "CPL",
    // 0x30
    "",
    "",
    "LD   (HL-), A",
    "INC  SP",
    "INC  (HL)",
    "DEC  (HL)",
    "",
    "SCF",
    // 0x38
    "",
    "ADD  HL, SP",
    "LD   A, (HL-)",
    "DEC  SP",
    "INC  A",
    "DEC  A",
    "",
    "CCF",
    // 0x40
    "LD   B, B",
    "LD   B, C",
    "LD   B, D",
    "LD   B, E",
    "LD   B, H",
    "LD   B, L",
    "LD   B, (HL)",
    "LD   B, A",
    // 0x48
    "LD   C, B",
    "LD   C, C",
    "LD   C, D",
    "LD   C, E",
    "LD   C, H",
    "LD   C, L",
    "LD   C, (HL)",
    "LD   C, A",
    // 0x50
    "LD   D, B",
    "LD   D, C",
    "LD   D, D",
    "LD   D, E",
    "LD   D, H",
    "LD   D, L",
    "LD   D, (HL)",
    "LD   D, A",
    // 0x58
    "LD   E, B",
    "LD   E, C",
    "LD   E, D",
    "LD   E, E",
    "LD   E, H",
    "LD   E, L",
    "LD   E, (HL)",
    "LD   E, A",
    // 0x60
    "LD   H, B",
    "LD   H, C",
    "LD   H, D",
    "LD   H, E",
    "LD   H, H",
    "LD   H, L",
    "LD   H, (HL)",
    "LD   H, A",
    // 0x68
    "LD   L, B",
    "LD   L, C",
    "LD   L, D",
    "LD   L, E",
    "LD   L, H",
    "LD   L, L",
    "LD   L, (HL)",
    "LD   L, A",
    // 0x70
    "LD   (HL), B",
    "LD   (HL), C",
    "LD   (HL), D",
    "LD   (HL), E",
    "LD   (HL), H",
    "LD   (HL), L",
    "HALT",
    "LD   (HL), A",
    // 0x78
    "LD   A, B",
    "LD   A, C",
    "LD   A, D",
    "LD   A, E",
    "LD   A, H",
    "LD   A, L",
    "LD   A, (HL)",
    "LD   A, A",
    // 0x80
    "ADD  A, B",
    "ADD  A, C",
    "ADD  A, D",
    "ADD  A, E",
    "ADD  A, H",
    "ADD  A, L",
    "ADD  A, (HL)",
    "ADD  A, A",
    // 0x88
    "ADC  A, B",
    "ADC  A, C",
    "ADC  A, D",
    "ADC  A, E",
    "ADC  A, H",
    "ADC  A, L",
    "ADC  A, (HL)",
    "ADC  A, A",
    // 0x90
    "SUB  B",
    "SUB  C",
    "SUB  D",
    "SUB  E",
    "SUB  H",
    "SUB  L",
    "SUB  (HL)",
    "SUB  A",
    // 0x98
    "SBC  A, B",
    "SBC  A, C",
    "SBC  A, D",
    "SBC  A, E",
    "SBC  A, H",
    "SBC  A, L",
    "SBC  A, (HL)",
    "SBC  A, A",
    // 0xA0
    "AND  B",
    "AND  C",
    "AND  D",
    "AND  E",
    "AND  H",
    "AND  L",
    "AND  (HL)",
    "AND  A",
    // 0xA8
    "XOR  B",
    "XOR  C",
    "XOR  D",
    "XOR  E",
    "XOR  H",
    "XOR  L",
    "XOR  (HL)",
    "XOR  A",
    // 0xB0
    "OR   B",
    "OR   C",
    "OR   D",
    "OR   E",
    "OR   H",
    "OR   L",
    "OR   (HL)",
    "OR   A",
    // 0xB8
    "CP   B",
    "CP   C",
    "CP   D",
    "CP   E",
    "CP   H",
    "CP   L",
    "CP   (HL)",
    "CP   A",
    // 0xC0
    "RET  NZ",
    "POP  BC",
    "",
    "",
    "",
    "PUSH BC",
    "",
    "RST  00H",
    // 0xC8
    "RET  Z",
    "RET",
    "",
    "",
    "",
    "",
    "",
    "RST  08H",
    // 0xD0
    "RET  NC",
    "POP  DE",
    "",
    "",
    "",
    "PUSH DE",
    "",
    "RST  10H",
    // 0xD8
    "RET  C",
    "RETI",
    "",
    "",
    "",
    "",
    "",
    "RST  18H",
    // 0xE0
    "",
    "POP  HL",
    "LD   (C), A",
    "",
    "",
    "PUSH HL",
    "",
    "RST  20H",
    // 0xE8
    "",
    "JP   (HL)",
    "",
    "",
    "",
    "",
    "",
    "RST  28H",
    // 0xF0
    "",
    "POP  AF",
    "LD   A, (C)",
    "DI",
    "",
    "PUSH AF",
    "",
    "RST 30H",
    // 0xF8
    "",
    "LD   SP, HL",
    "",
    "EI",
    "",
    "",
    "",
    "RST  38H",
];

pub fn format_mnemonic(mmu: &MMU, addr: u16) -> String {
    let op: u8 = mmu.direct_read(addr);

    match op {
        0x01 => format!("LD  BC, ${:04X}", mmu.direct_read_u16(addr + 1)),

        // LD n, d: load immediate into register n
        0x06 => format!("LD   B, ${:02X}", mmu.direct_read(addr + 1)),
        0x08 => format!("LD   ${:02X}, SP", mmu.direct_read(addr + 1)),
        0x0E => format!("LD   C, ${:02X}", mmu.direct_read(addr + 1)),
        0x16 => format!("LD   D, ${:02X}", mmu.direct_read(addr + 1)),
        0x1E => format!("LD   E, ${:02X}", mmu.direct_read(addr + 1)),
        0x26 => format!("LD   H, ${:02X}", mmu.direct_read(addr + 1)),
        0x2E => format!("LD   L, ${:02X}", mmu.direct_read(addr + 1)),
        0x3E => format!("LD   A, ${:02X}", mmu.direct_read(addr + 1)),

        0x11 => {
            let lo = mmu.direct_read(addr + 1);
            let hi = mmu.direct_read(addr + 2);
            format!("LD   DE, ${:02X}{:02X}", hi, lo)
        }

        0x18 => {
            let rel = mmu.direct_read_i8(addr + 1);
            let abs = add_i8_to_u16(addr + 2, rel);
            format!("JR   {}  ; jump to 0x{:04X}", rel, abs)
        }

        0x1A => {
            let de = mmu.reg.de();
            let val = mmu.direct_read(de);
            format!("LD   A, (DE)  ; DE=0x{:04X} (DE)=0x{:02X}", de, val)
        }

        0x20 => {
            let rel = mmu.direct_read_i8(addr + 1);
            let abs = add_i8_to_u16(addr + 2, rel);
            format!("JR   NZ, {}    ; jump to 0x{:04X}", rel, abs)
        }

        0x21 => {
            let lo = mmu.direct_read(addr + 1);
            let hi = mmu.direct_read(addr + 2);
            format!("LD   HL, ${:02X}{:02X}", hi, lo)
        }

        0x28 => {
            let rel = mmu.direct_read_i8(addr + 1);
            let abs = add_i8_to_u16(addr + 2, rel);
            format!("JR   Z, {}        ; jump to 0x{:04X}", rel, abs)
        }

        0x30 => {
            let rel = mmu.direct_read_i8(addr + 1);
            let abs = add_i8_to_u16(addr + 2, rel);
            format!("JR   NC, {}    ; jump to 0x{:04X}", rel, abs)
        }

        0x31 => {
            let lo = mmu.direct_read(addr + 1);
            let hi = mmu.direct_read(addr + 2);
            format!("LD   SP, ${:02X}{:02X}", hi, lo)
        }

        0xC2 => format!("JP   NZ, 0x{:04X}", mmu.direct_read_u16(addr + 1)),
        0xC3 => format!("JP   0x{:04X}", mmu.direct_read_u16(addr + 1)),
        0xC4 => format!("CALL  NZ, ${:04X}", mmu.direct_read_u16(addr + 1)),

        0xCA => format!("JP   Z, 0x{:04X}", mmu.direct_read_u16(addr + 1)),

        0xCB => {
            let op2 = mmu.direct_read(addr + 1);
            match op2 {
                0x11 => "RL   C".to_string(),
                0x7C => "BIT 7, h".to_string(),
                0x37 => "SWAP A".to_string(),
                0x7E => "SET 4, A".to_string(),
                0xE6 => "SET 4, (HL)".to_string(),
                0xEE => "SET 5, (HL)".to_string(),
                _ => {
                    panic!("invalid instruction op code: 0x{:02X}{:02X}", op, op2);
                }
            }
        }

        0xBE => format!(
            "CP   (HL)  ; HL=0x{:04X} (HL)=0x{:02X}",
            mmu.reg.hl(),
            mmu.direct_read(mmu.reg.hl())
        ),

        0xCD => format!("CALL ${:04X}", mmu.direct_read_u16(addr + 1)),
        0xCE => format!("ADC  A, 0x{:02X}", mmu.direct_read(addr + 1)),

        0xD2 => format!("JP   NC, 0x{:04X}", mmu.direct_read_u16(addr + 1)),
        0xD6 => format!("SUB  0x{:02X}", mmu.direct_read(addr + 1)),

        0xE0 => format!("LD   ($FF00+${:02X}), A", mmu.direct_read(addr + 1)),
        0xEA => format!("LD   (${:04X}), A", mmu.direct_read_u16(addr + 1)),
        0xE6 => format!("AND  ${:02X}", mmu.direct_read(addr + 1)),

        0xF0 => format!("LD   A, ($FF00+${:02X})", mmu.direct_read(addr + 1)),
        0xFA => format!("LD   A, (${:04X})", mmu.direct_read_u16(addr + 1)),
        0xF8 => format!("LD   HL, SP + ${:02X}", mmu.direct_read(addr + 1)),
        0xFE => format!("CP   ${:02X}", mmu.direct_read(addr + 1)),

        0xFC => format!("! Illegal op code: 0x{:02X}", op),

        _ => {
            let easy = SIMPLE_MNEMONICS[op as usize];

            if !easy.is_empty() {
                return easy.to_string();
            }

            panic!(
                "invalid instruction op code at 0x{:04X}: 0x{:02X}",
                addr, op
            );
        }
    }
}

pub fn print_listing(mmu: &MMU, addr: u16, line_count: i32) -> u16 {
    let mut a = addr;
    for _n in 0..line_count {
        println!("0x{:04X}: {}", a, format_mnemonic(&mmu, a));
        a = a + (op_length(mmu.direct_read(addr)) as u16);
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
