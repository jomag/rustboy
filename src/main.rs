extern crate clap;
extern crate ctrlc;
extern crate num_traits;
extern crate png;
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
use sdl2::keyboard::Keycode;
use sdl2::event::Event;

#[macro_escape]
mod macros;
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
mod buttons;
mod cartridge;

use debug::{format_mnemonic, print_listing, print_registers, print_sprites, print_lcdc};
use emu::Emu;
use lcd::{LCD, SCREEN_HEIGHT, SCREEN_WIDTH};
use buttons::ButtonType;

const APPNAME: &str = "Rustboy?";
const VERSION: &str = "0.0.0";
const AUTHOR: &str = "Jonatan Magnusson <jonatan.magnusson@gmail.com>";
const BOOTSTRAP_ROM: &str = "rom/boot.gb";
const CARTRIDGE_ROM: &str = "rom/tetris.gb";
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH * 2) as u32;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT * 2) as u32;

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

fn parse_number<T: num_traits::Num>(text: &str) -> Result<T, T::FromStrRadixErr> {
    if text.starts_with("0x") {
        T::from_str_radix(&text[2..], 16)
    } else {
        T::from_str_radix(text, 10)
    }
}

fn parse_optional<T: num_traits::Num>(value: Option<&str>) -> Option<T> {
    match value {
        Some(text) => match parse_number(text) {
            Ok(num) => Some(num),
            Err(_) => {
                println!("Not a valid number: {:?}", text);
                std::process::exit(1);
            }
        },
        None => None,
    }
}

fn parse<T: num_traits::Num>(value: Option<&str>, default: T) -> T {
    match parse_optional(value) {
        Some(num) => num,
        None => default,
    }
}

fn capture_frame(filename: &str, frame: u32, lcd: &LCD) -> Result<(), std::io::Error> {
    // For reading and opening files
    use std::fs::File;
    use std::io::BufWriter;
    use std::path::Path;

    // To use encoder.set()
    use png::HasParameters;

    let path = Path::new(filename);
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&lcd.buf_rgb8).unwrap();

    println!("Captured frame {}", frame);
    return Ok(());
}

fn main() -> Result<(), String> {
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
            --capture-to=[FILE] 'Capture filename'
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
    let capture_at_frame: Option<u32> = parse_optional(matches.value_of("capture"));
    let capture_filename: &str = matches
        .value_of("capture-to")
        .unwrap_or("capture-frame-#.png");

    let mut emu = Emu::new();
    emu.init();

    println!("Loading bootstrap ROM: {}", bootstrap_rom);
    let sz = emu.load_bootstrap(bootstrap_rom);
    println!(" - {} bytes read", sz);

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
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rustboy", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position(100, 100)
        .opengl()
        .build()
        .map_err(|msg| msg.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let fmt = PixelFormatEnum::RGB24;

    let mut texture = texture_creator
        .create_texture_streaming(fmt, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .map_err(|e| e.to_string())?;

    canvas.clear();
    canvas.copy(&texture, None, Some(Rect::new(150, 150, 320, 288)))?;
    canvas.present();

    let mut event_pump = sdl_context.event_pump().map_err(|msg| msg.to_string())?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },

                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },

                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::Left => emu.mmu.buttons.handle_press(ButtonType::Left),
                        Keycode::Right => emu.mmu.buttons.handle_press(ButtonType::Right),
                        Keycode::Up => emu.mmu.buttons.handle_press(ButtonType::Up),
                        Keycode::Down => emu.mmu.buttons.handle_press(ButtonType::Down),
                        Keycode::Space => emu.mmu.buttons.handle_press(ButtonType::Select),
                        Keycode::Return => emu.mmu.buttons.handle_press(ButtonType::Start),
                        Keycode::Z => emu.mmu.buttons.handle_press(ButtonType::A),
                        Keycode::X => emu.mmu.buttons.handle_press(ButtonType::B),
                        _ => {}
                    }
                },

                Event::KeyUp { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::Left => emu.mmu.buttons.handle_release(ButtonType::Left),
                        Keycode::Right => emu.mmu.buttons.handle_release(ButtonType::Right),
                        Keycode::Up => emu.mmu.buttons.handle_release(ButtonType::Up),
                        Keycode::Down => emu.mmu.buttons.handle_release(ButtonType::Down),
                        Keycode::Space => emu.mmu.buttons.handle_release(ButtonType::Select),
                        Keycode::Return => emu.mmu.buttons.handle_release(ButtonType::Start),
                        Keycode::Z => emu.mmu.buttons.handle_release(ButtonType::A),
                        Keycode::X => emu.mmu.buttons.handle_release(ButtonType::B),
                        _ => {}
                    }
                },

                _ => {
                    // println!("unhandled event: {:?}", event);
                }
            }
            /*
            match event {
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
                    "break" => {
                        match parse_number::<u16>(args[1]) {
                            Ok(addr) => breakpoints.push(addr),
                            Err(_) => println!("Not a valid address"),
                        };
                    }
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
                    "r" => {
                        if args.len() > 1 {
                            match parse_number::<u16>(args[1]) {
                                Ok(addr) => println!(
                                    "[{:04X}] = 0x{:02X}",
                                    addr,
                                    &emu.mmu.direct_read(addr),
                                ),
                                Err(_) => println!("Not a valid address"),
                            };
                        }
                    }
                    "sprites" => {
                        print_sprites(&emu.mmu);
                    }
                    "lcdc" => print_lcdc(&emu.mmu),
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
            if let Some(frm) = capture_at_frame {
                if frm == frame_counter {
                    capture_frame(capture_filename, frame_counter, &emu.mmu.lcd).unwrap();
                }
            }

            /*
            let mut texture = texture_creator
                .create_texture_streaming(fmt, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
                .unwrap();
                */

            texture
                .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                    buffer.copy_from_slice(&emu.mmu.lcd.buf_rgb8);
                })
                .unwrap();

            canvas.clear();

            // FIXME: there's currently a problem with the next statement:
            // On MacOS the second parameter for React::new should be
            // "WINDOW_HEIGHT as i32", while on Linux it should be 0.
            canvas
                .copy(
                    &texture,
                    None,
                    Rect::new(0, 0 /*WINDOW_HEIGHT as i32*/, WINDOW_WIDTH, WINDOW_HEIGHT),
                )
                .unwrap();

            canvas.present();

            if let Some(frm) = exit_at_frame {
                if frm == frame_counter {
                    println!("Exit at frame {}", frame_counter);
                    break 'running;
                }
            }

            emu.mmu.display_updated = false;
            frame_counter = frame_counter + 1;
        }
    }

    println!("Clean shutdown. Bye!");
    return Ok(());
}
