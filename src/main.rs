
extern crate ctrlc;
extern crate sdl2;

use std::io::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{ AtomicBool, Ordering };
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

mod ui;
mod registers;
mod memory;
mod instructions;
mod debug;
mod lcd;
mod sound;
mod timer;

use debug::{ print_listing, print_registers, format_mnemonic };
use memory::Memory;
use registers::Registers;
use lcd::LCD;
use sound::{ sound_test };
use timer::Timer;

fn main() {
    sound_test();
    return;

    use std::io::stdin;
    use std::io::stdout;

    let mut reg = Registers::new();
    let mut mem = Memory::new();
    let mut lcd = LCD::new();
    let mut timer = Timer::new();

    println!();
    println!("Starting RustBoy (GameBoy Emulator written in Rust)");
    println!("---------------------------------------------------");
    println!();

    mem.load_bootstrap("rom/boot.gb");
    mem.load_cartridge("rom/tetris.gb");

    for a in 0x104..0x133 {
        print!("{:x},", mem.read(a));
    }

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
    // breakpoints.push(0x95);
    breakpoints.push(0x100);

    let ctrlc_event = Arc::new(AtomicBool::new(false));
    let ctrlc_event_clone = ctrlc_event.clone();

    ctrlc::set_handler(move || {
        println!("Ctrl-C: breaking execution");
        ctrlc_event_clone.store(true, Ordering::SeqCst)
    }).expect("failed to setup ctrl-c handler");

    let mut cycles: u32 = 0;

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

        if ctrlc_event.load(Ordering::SeqCst) {
            stepping = true;
        }

        if breakpoints.contains(&reg.pc) {
            println!("- at breakpoint (PC: 0x{:04X})", reg.pc);
            stepping = true;
        }

        if stepping {
            print_registers(&reg);
            let pc = reg.pc;
            let mut list_offset = pc;
            println!("0x{:04X}: {}", pc, format_mnemonic(&mem, pc));

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
                        list_offset = print_listing(&mem, list_offset, 10);
                    }
                    "q" => { break 'running; }
                    "" => {}
                    _ => { println!("invalid command!"); }
                }
            }
        }

        let op_cycles = instructions::step(&mut reg, &mut mem);
        cycles += op_cycles;

        timer.update(&mut mem, op_cycles);

        let refresh = lcd.update(op_cycles, &mut mem, &mut texture);

        if refresh {
            lcd.copy_to_texture(&mut texture);
            canvas.clear();
            canvas.copy(&texture, None, Some(Rect::new(2, 2, 320, 288))).unwrap();
            canvas.present();
        }
    }

    println!("Clean shutdown. Bye!");
}

