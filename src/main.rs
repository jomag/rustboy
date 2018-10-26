
extern crate ctrlc;
extern crate sdl2;

use std::io::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{ AtomicBool, Ordering };
use std::io::stdin;
use std::io::stdout;
use std::env;

mod ui;
mod registers;
mod mmu;
mod instructions;
mod debug;
mod lcd;
mod sound;
mod timer;
mod interrupt;
mod cpu;
mod dma;
mod emu;

use debug::{ print_listing, print_registers, format_mnemonic };
use emu::EmuSDL;

fn main() {
    let bootstrap_rom ="rom/boot.gb";

    let args: Vec<String> = env::args().collect();

    let cartridge_rom = if args.len() > 1 { &args[1] } else { "rom/tetris.gb" };

    /*
    let state_log = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("log.txt")?;
        */

    /*
    let mut state_log = match File::create("log.txt") {
        Err(reason) => { panic!("Failed to create log"); },
        Ok(file) => { file } 
    };
    */

    // sound_test();
    // return;

    
    println!();
    println!("Starting RustBoy (GameBoy Emulator written in Rust)");
    println!("---------------------------------------------------");
    println!();

    let emu = EmuSDL::new();
    emu.init();

    println!("Loading bootstrap ROM: {}", bootstrap_rom);
    emu.load_bootstrap(bootstrap_rom);

    println!("Loading cartridge ROM: {}", cartridge_rom);
    emu.load_cartridge(cartridge_rom);

    let mut breakpoints: Vec<u16> = Vec::new();
    let mut stepping = false;
    let mut last_command = "".to_string();

    // breakpoints.push(0x000C);
    // breakpoints.push(0x0034);
    // breakpoints.push(0x0040);
    // breakpoints.push(0x0051);
    // breakpoints.push(0x6A);
    // breakpoints.push(0x27);
    breakpoints.push(0x150);
    //breakpoints.push(0x55);
    //breakpoints.push(0x6A);
    // breakpoints.push(0x40);
    // breakpoints.push(0x2A02);
    // breakpoints.push(0x2A18);

    stepping = false;

    let ctrlc_event = Arc::new(AtomicBool::new(false));
    let ctrlc_event_clone = ctrlc_event.clone();

    ctrlc::set_handler(move || {
        println!("Ctrl-C: breaking execution");
        ctrlc_event_clone.store(true, Ordering::SeqCst)
    }).expect("failed to setup ctrl-c handler");

    let mut cycles: u32 = 0;

    let mut op_usage: [u32; 256] = [0; 256];


    'running: loop {
        /* THIS SLOWS DOWN THE CODE! NOT SURE WHY!*/
        /*
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => { println!("unhandled event"); }
            }
        }
        */

        /*
        if cpu.mem.mem[cpu.reg.pc as usize] == 0xFB {
            stepping = true;
        }
        if cpu.mem.mem[cpu.reg.pc as usize] == 0xF3 {
            stepping = true;
        }
        if cpu.mem.mem[cpu.reg.pc as usize] == 0xCA {
            stepping = true;
        }
        */

        if emu.mmu.reg.pc > 0x5000 { 
            //stepping = true;
        }

        if ctrlc_event.load(Ordering::SeqCst) {
            ctrlc_event.store(false, Ordering::SeqCst);
            stepping = true;
        }

        if breakpoints.contains(&emu.mmu.reg.pc) {
            println!("- at breakpoint (PC: 0x{:04X})", emu.mmu.reg.pc);
            stepping = true;
        }

        /*
        if mmu.cpu.mem.watch_triggered {
            println!("Break: watched memory change (PC 0x{:04X})", cpu.reg.pc);
            cpu.mem.watch_triggered = false;
            stepping = true;
        }
        */

        if stepping {
            print_registers(&emu.mmu);
            let pc = emu.mmu.reg.pc;
            let mut list_offset = pc;
            println!("0x{:04X}: {}", pc, format_mnemonic(&emu.mmu, pc));

            loop {
                print!("(debug) ");
                stdout().flush().ok();
                let mut cmd_s: String = String::new();
                stdin().read_line(&mut cmd_s).expect("invalid command");

                if cmd_s == "" {
                    cmd_s = last_command.clone();
                } else {
                    last_command = cmd_s.clone();
                }

                let args: Vec<_> = cmd_s.split_whitespace().collect();

                match args[0] {
                    "c" => { stepping = false; break; },
                    "s" => { break; }
                    "n" => { break; }
                    "l" => {
                        if args.len() > 1 {
                            list_offset = args[1].parse::<u16>().unwrap();
                        }
                        list_offset = print_listing(&emu.mmu, list_offset, 10);
                    }
                    "q" => { break 'running; }
                    "" => {}
                    _ => { println!("invalid command!"); }
                }
            }
        }

        if emu.mmu.reg.stopped {
            println!("Stopped! Press enter to continue");
            let mut inp: String = String::new();
            stdin().read_line(&mut inp).expect("invalid command");
            emu.mmu.reg.stopped = false;
        }

        emu.step();
    }

    println!("Clean shutdown. Bye!");
}

