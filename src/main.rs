
// use std::io;
use std::io::prelude::*;
use std::fs::File;

fn print_hex(buf: &Vec<u8>) {
    let len = buf.len();

    for i in (0..len).step_by(16) {
        print!("{:04x}: ", i);
        for j in i..i+16 {
            print!("{:02x} ", buf[j]);
        }
        println!("");
    }
}

fn main() {
    let mut reg_a: u8 = 0;
    let mut reg_b: u8 = 0;
    let mut reg_c: u8 = 0;
    let mut reg_d: u8 = 0;
    let mut reg_e: u8 = 0;
    let mut reg_f: u8 = 0;
    let mut reg_h: u8 = 0;
    let mut reg_l: u8 = 0;
    let mut reg_sp: u16 = 0;
    let mut reg_pc: u16 = 0;
    let mut mem: [u8; 0xFFFF] = [0; 0xFFFF];

    println!();
    println!("Starting RustBoy (GameBoy Emulator written in Rust)");
    println!("---------------------------------------------------");
    println!();

    // Open and read content of boot rom
    let filename = "rom/boot.gb";
    let mut f = File::open(filename)
        .expect("failed to open boot rom");
    let mut boot_rom = Vec::new();
    f.read_to_end(&mut boot_rom)
        .expect("failed to read content of boot rom");

    println!("buffer length: {}", boot_rom.len());
    print_hex(&boot_rom);

    loop {
        let ptr = reg_pc as usize;
        let op: u8 = boot_rom[ptr];

        match op {
            0x01 => {
                reg_b = boot_rom[ptr + 2];
                reg_c = boot_rom[ptr + 1];
                println!("LD  BC, ${:04X}", (reg_b as u16) << 8 + reg_c);
                reg_pc += 3;
            }
            0x11 => {
                reg_d = boot_rom[ptr + 2];
                reg_e = boot_rom[ptr + 1];
                println!("LD  DE, ${:04X}", (reg_d as u16) << 8 + reg_e);
                reg_pc += 3;
            }
            0x21 => {
                reg_h = boot_rom[ptr + 2];
                reg_l = boot_rom[ptr + 1];
                println!("LD  HL, ${:04X}", ((reg_h as u16) << 8) + (reg_l as u16));
                reg_pc += 3;
            }
            0x31 => {
                reg_sp = ((boot_rom[ptr + 2] as u16) << 8) | (boot_rom[ptr + 1] as u16);
                println!("LD  SP, ${:04X}", reg_sp);
                reg_pc += 3;
            }
            0xAF => {
                reg_a |= reg_a ^ reg_a;
                println!("XOR A");
                reg_pc += 1;
            }
            0xA8 => {
                reg_a |= reg_a ^ reg_b;
                println!("XOR B");
                reg_pc += 1;
            }
            0xA9 => {
                reg_a |= reg_a ^ reg_c;
                println!("XOR C");
                reg_pc += 1;
            }
            0xAA => {
                reg_a |= reg_a ^ reg_d;
                println!("XOR D");
                reg_pc += 1;
            }
            0xAB => {
                reg_a |= reg_a ^ reg_e;
                println!("XOR E");
                reg_pc += 1;
            }
            0xAC => {
                reg_a |= reg_a ^ reg_h;
                println!("XOR H");
                reg_pc += 1;
            }
            0xAD => {
                reg_a |= reg_a ^ reg_l;
                println!("XOR L");
                reg_pc += 1;
            }
            0x32 => {
                println!("LDD (HL), A");
                let hl: u16 = ((reg_h as u16) << 8) | (reg_l as u16);
                mem[hl as usize] = reg_a;
                reg_pc +=1;
            }
            0xCB => {
                let op2 = boot_rom[ptr + 1];
                println!("SECOND OP: {:x}", op2);
            }
            _ => {
                println!("Unsupported opcode: 0x{:X}", op);
                break;
            }
        }
    }
    
    
    println!("nothing more to do! bye!");
}


