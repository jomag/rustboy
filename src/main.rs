extern crate clap;
extern crate ctrlc;
extern crate sdl2;
extern crate serde;

use std::env;
use std::io::prelude::*;
use std::io::stdin;
use std::io::stdout;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

mod cpu;
mod debug;
mod dma;
mod emu;
mod instructions;
mod interrupt;
mod lcd;
mod mmu;
mod registers;
mod timer;
mod ui;

use debug::{format_mnemonic, print_listing, print_registers};
use emu::Emu;
use lcd::{SCREEN_HEIGHT, SCREEN_WIDTH, LCD};

const APPNAME: &str = "Rustboy?";
const VERSION: &str = "0.0.0";
const AUTHOR: &str = "Jonatan Magnusson <jonatan.magnusson@gmail.com>";
const BOOTSTRAP_ROM: &str = "rom/boot.gb";
const CARTRIDGE_ROM: &str = "rom/tetris.gb";

fn should_enter_stepping(emu: &mut Emu, breakpoints: &Vec<u16>) -> bool {
    if emu.mmu.timer.trigger_debug {
        emu.mmu.timer.trigger_debug = false;
        return true;
    }

    if breakpoints.contains(&emu.mmu.reg.pc) {
        println!("- at breakpoint (PC: 0x{:04X})", emu.mmu.reg.pc);
        return true;
    }

    if emu.mmu.watch_triggered {
        println!("Break: watched memory change (PC 0x{:04X})", emu.mmu.reg.pc);
        emu.mmu.watch_triggered = false;
        return true;
    }

    return false;
}

fn parse_optional<T: FromStr>(value: Option<&str>) -> Option<T> {
    match value {
        Some(num) => match num.parse::<T>() {
            Ok(num) => Some(num),
            Err(_) => {
                println!("Not a valid number: {:?}", num);
                std::process::exit(1);
            }
        },
        None => None,
    }
}

fn parse<T: FromStr>(value: Option<&str>, default: T) -> T {
    match parse_optional(value) {
        Some(num) => num,
        None => default,
    }
}

fn capture_frame(lcd: LCD) -> Result<(), E> {
    
}

fn main() {
    let matches = clap::App::new(APPNAME)
        .version(VERSION)
        .author(AUTHOR)
        .about("Your less than average GameBoy emulator")
        .args_from_usage(
            "<ROM>              'The ROM to run'
            -B, --boot=[FILE]   'Path to bootstrap ROM'
            -H, --headless      'Run headless'
            -b, --break=[ADDR]  'Break at address ADDR'
            --break-cycle=[N]   'Break at cycle N'
            --break-frame=[N]   'Break at frame N'
            --exit-cycle=[N]    'Exit at cycle N'
            --exit-frame=[N]    'Exit at frame N'
            -R, --record=[PATH] 'Record into directory'
            -s, --skip=[N]      'Frames to skip while recording'
            -C, --capture=[N]   'Capture screen at frame N'
            ",
        )
        .get_matches();

    let bootstrap_rom = matches.value_of("boot").unwrap_or(BOOTSTRAP_ROM); // done!
    let cartridge_rom = matches.value_of("ROM").unwrap_or(CARTRIDGE_ROM); // done!
    let headless: bool = matches.is_present("headless");
    let record: Option<&str> = matches.value_of("record");
    let record_frame_skip: u32 = parse(matches.value_of("skip"), 3);
    let break_at_address: Option<u16> = parse_optional(matches.value_of("break"));
    let break_at_cycle: Option<u64> = parse_optional(matches.value_of("break-cycle"));
    let break_at_frame: Option<u32> = parse_optional(matches.value_of("break-frame"));
    let exit_at_cycle: Option<u32> = parse_optional(matches.value_of("exit-cycle"));
    let exit_at_frame: Option<u32> = parse_optional(matches.value_of("exit-frame"));
    let capture_frame: Option<u32> = parse_optional(matches.value_of("capture"));

    let mut emu = Emu::new();
    emu.init();

    println!("Loading bootstrap ROM: {}", bootstrap_rom);
    emu.load_bootstrap(bootstrap_rom);

    println!("Loading cartridge ROM: {}", cartridge_rom);
    emu.load_cartridge(cartridge_rom);

    let mut breakpoints: Vec<u16> = Vec::new();
    let mut stepping = false;
    let mut last_command = "".to_string();

    if let Some(addr) = break_at_address {
        breakpoints.push(addr);
    }

    if let Some(cycle) = break_at_cycle {
        emu.mmu.timer.abs_cycle_breakpoint = cycle;
    }

    let ctrlc_event = Arc::new(AtomicBool::new(false));
    let ctrlc_event_clone = ctrlc_event.clone();

    ctrlc::set_handler(move || {
        println!("Ctrl-C: breaking execution");
        ctrlc_event_clone.store(true, Ordering::SeqCst)
    })
    .expect("failed to setup ctrl-c handler");

    let mut frame_counter: u32 = 0;

    // Setup SDL2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("rustboy", 320, 288)
        .position(100, 100)
        .opengl()
        .build()
        .map_err(|msg| println!("SDL2 failure: {}", msg))
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| println!("error: {}", e))
        .unwrap();
    let texture_creator = canvas.texture_creator();
    let fmt = PixelFormatEnum::RGB24;
    let mut texture = texture_creator
        .create_texture_streaming(fmt, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .unwrap();

    canvas.clear();
    canvas
        .copy(&texture, None, Some(Rect::new(150, 150, 320, 288)))
        .unwrap();
    canvas.present();

    let mut event_pump = sdl_context
        .event_pump()
        .map_err(|msg| println!("SDL2 event pump failure: {}", msg))
        .unwrap();

    'running: loop {
        /* THIS SLOWS DOWN THE CODE! NOT SURE WHY!*/
        for event in event_pump.poll_iter() {
            match event {
                _ => {
                    println!("unhandled event");
                }
            }
            /*
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => { println!("unhandled event"); }
            }*/
        }

        if should_enter_stepping(&mut emu, &breakpoints) {
            stepping = true;
        }

        if ctrlc_event.load(Ordering::SeqCst) {
            ctrlc_event.store(false, Ordering::SeqCst);
            stepping = true;
        }

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
                    "c" => {
                        stepping = false;
                        break;
                    }
                    "s" => {
                        break;
                    }
                    "n" => {
                        break;
                    }
                    "l" => {
                        if args.len() > 1 {
                            list_offset = args[1].parse::<u16>().unwrap();
                        }
                        list_offset = print_listing(&emu.mmu, list_offset, 10);
                    }
                    "q" => {
                        break 'running;
                    }
                    "" => {}
                    _ => {
                        println!("invalid command!");
                    }
                }
            }
        }

        if emu.mmu.reg.stopped {
            println!("Stopped! Press enter to continue");
            let mut inp: String = String::new();
            stdin().read_line(&mut inp).expect("invalid command");
            emu.mmu.reg.stopped = false;
        }

        emu.mmu.exec_op();

        while !stepping && !emu.mmu.display_updated {
            if should_enter_stepping(&mut emu, &breakpoints) {
                stepping = true;
            } else {
                emu.mmu.exec_op();
            }
        }

        if emu.mmu.display_updated {
            canvas.clear();

            if let Some(frm) = capture_frame && frm == frame_counter {
                capture_frame(capture_filename, &emu.mmu.lcd);
            }

            let mut texture = texture_creator
                .create_texture_streaming(fmt, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
                .unwrap();

            texture
                .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                    buffer.copy_from_slice(&emu.mmu.lcd.buf_rgb8);
                })
                .unwrap();

            canvas
                .copy(&texture, None, Rect::new(0, 288, 320, 288))
                .unwrap();
            canvas.present();
            emu.mmu.display_updated = false;
            frame_counter = frame_counter + 1;
        }
    }

    println!("Clean shutdown. Bye!");
}
