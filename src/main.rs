
extern crate ctrlc;
extern crate sdl2;

use std::io::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{ AtomicBool, Ordering };
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::io::stdin;
use std::io::stdout;
use std::env;
use std::fs::File;

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

use debug::{ print_listing, print_registers, format_mnemonic, log_state };
use registers::Registers;
use lcd::LCD;
use sound::{ sound_test };
use timer::Timer;
use interrupt::handle_interrupts;
use mmu::MMU;

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
    let mut state_log = match File::create("log.txt") {
        Err(reason) => { panic!("Failed to create log"); },
        Ok(file) => { file } 
    };

    // sound_test();
    // return;

    let mut mmu = MMU::new();
    mmu.init();

    println!();
    println!("Starting RustBoy (GameBoy Emulator written in Rust)");
    println!("---------------------------------------------------");
    println!();

    println!("Loading bootstrap ROM: {}", bootstrap_rom);
    mmu.load_bootstrap(bootstrap_rom);

    println!("Loading cartridge ROM: {}", cartridge_rom);
    mmu.load_cartridge(cartridge_rom);

    let mut breakpoints: Vec<u16> = Vec::new();
    let mut stepping = false;
    let mut last_command = "".to_string();

    // Initialize UI
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rustboy", 320 + 4, 288 + 4)
        // .position_centered()
        .position(100, 100)
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let fmt = PixelFormatEnum::RGB24;
    let mut texture = texture_creator.create_texture_streaming(fmt, 160, 144).unwrap();

    canvas.clear();
    canvas.copy(&texture, None, Some(Rect::new(2, 2, 320, 288))).unwrap();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    // breakpoints.push(0x000C);
    // breakpoints.push(0x0034);
    // breakpoints.push(0x0040);
    // breakpoints.push(0x0051);
    // breakpoints.push(0x6A);
    // breakpoints.push(0x27);
    breakpoints.push(0x100);
    // breakpoints.push(0x40);
    // breakpoints.push(0x2A02);
    // breakpoints.push(0x2A18);

    stepping = true;

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

        if mmu.reg.pc > 0x5000 { 
            //stepping = true;
        }

        if ctrlc_event.load(Ordering::SeqCst) {
            ctrlc_event.store(false, Ordering::SeqCst);
            stepping = true;
        }

        if breakpoints.contains(&mmu.reg.pc) {
            println!("- at breakpoint (PC: 0x{:04X})", mmu.reg.pc);
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
            print_registers(&mmu);
            let pc = mmu.reg.pc;
            let mut list_offset = pc;
            println!("0x{:04X}: {}", pc, format_mnemonic(&mmu, pc));

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
                        list_offset = print_listing(&mmu, list_offset, 10);
                    }
                    "q" => { break 'running; }
                    "" => {}
                    _ => { println!("invalid command!"); }
                }
            }
        }

        if mmu.reg.stopped {
            println!("Stopped! Press enter to continue");
            let mut inp: String = String::new();
            stdin().read_line(&mut inp).expect("invalid command");
            mmu.reg.stopped = false;
        }

        // log_state(&mut state_log, &mmu);
        mmu.exec_op();

        if mmu.display_updated {
            mmu.display_updated = false;
            mmu.lcd.copy_to_texture(&mut texture);
            canvas.clear();
            canvas.copy(&texture, None, Some(Rect::new(2, 2, 320, 288))).unwrap();
            canvas.present();
        }
    }

    println!("Clean shutdown. Bye!");
}

