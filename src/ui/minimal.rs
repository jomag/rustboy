use std::{
    io::{stdin, stdout, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::{
    buttons::ButtonType,
    debug::{
        format_mnemonic, print_apu, print_lcdc, print_listing, print_registers, print_sprites,
    },
    emu::Emu,
    parse_number,
};
use lcd::{SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl2::{
    event::Event, keyboard::Keycode, pixels::PixelFormatEnum, rect::Rect, video::SwapInterval,
};

use super::{audio::SAMPLE_RATE, FPS};

const WINDOW_WIDTH: u32 = (SCREEN_WIDTH * 2) as u32;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT * 2) as u32;

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}

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

pub fn run_with_minimal_ui(
    title: &str,
    width: Option<u32>,
    height: Option<u32>,
    emu: &mut Emu,
) -> Result<(), String> {
    // Setup SDL2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            title,
            width.unwrap_or(WINDOW_WIDTH),
            height.unwrap_or(WINDOW_HEIGHT),
        )
        .position(100, 100)
        .opengl()
        .build()
        .map_err(|msg| msg.to_string())
        .unwrap();

    let mut breakpoints: Vec<u16> = Vec::new();
    let mut stepping = false;
    let mut last_command = "".to_string();

    let ctrlc_event = Arc::new(AtomicBool::new(false));
    let ctrlc_event_clone = ctrlc_event.clone();

    ctrlc::set_handler(move || {
        println!("Ctrl-C: breaking execution");
        ctrlc_event_clone.store(true, Ordering::SeqCst)
    })
    .expect("failed to setup ctrl-c handler");

    let mut frame_counter: u32 = 0;

    let mut canvas = window
        .into_canvas()
        .index(find_sdl_gl_driver().unwrap())
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let fmt = PixelFormatEnum::RGBA8888;

    let mut texture = texture_creator
        .create_texture_streaming(fmt, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .map_err(|e| e.to_string())?;

    video_subsystem.gl_set_swap_interval(SwapInterval::Immediate)?;
    canvas.clear();
    canvas.copy(&texture, None, Some(Rect::new(150, 150, 320, 288)))?;
    canvas.present();

    let mut event_pump = sdl_context.event_pump().map_err(|msg| msg.to_string())?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,

                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Left => emu.mmu.buttons.handle_press(ButtonType::Left),
                    Keycode::Right => emu.mmu.buttons.handle_press(ButtonType::Right),
                    Keycode::Up => emu.mmu.buttons.handle_press(ButtonType::Up),
                    Keycode::Down => emu.mmu.buttons.handle_press(ButtonType::Down),
                    Keycode::Space => emu.mmu.buttons.handle_press(ButtonType::Select),
                    Keycode::Return => emu.mmu.buttons.handle_press(ButtonType::Start),
                    Keycode::Z => emu.mmu.buttons.handle_press(ButtonType::A),
                    Keycode::X => emu.mmu.buttons.handle_press(ButtonType::B),
                    _ => {}
                },

                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Left => emu.mmu.buttons.handle_release(ButtonType::Left),
                    Keycode::Right => emu.mmu.buttons.handle_release(ButtonType::Right),
                    Keycode::Up => emu.mmu.buttons.handle_release(ButtonType::Up),
                    Keycode::Down => emu.mmu.buttons.handle_release(ButtonType::Down),
                    Keycode::Space => emu.mmu.buttons.handle_release(ButtonType::Select),
                    Keycode::Return => emu.mmu.buttons.handle_release(ButtonType::Start),
                    Keycode::Z => emu.mmu.buttons.handle_release(ButtonType::A),
                    Keycode::X => emu.mmu.buttons.handle_release(ButtonType::B),
                    _ => {}
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

        if should_enter_stepping(emu, &breakpoints) {
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
                    "apu" => print_apu(&emu.mmu),
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
            if should_enter_stepping(emu, &breakpoints) {
                stepping = true;
            } else {
                emu.mmu.exec_op();
            }
        }

        if emu.mmu.display_updated {
            // Generate one new frame worth of audio samples.
            // FIXME: i want to refactor this, so that the emulator
            // generates one sample for each cycle, store it in a buffer
            // and then downsamples to the selected sample rate. Until
            // then, audio is disabled.
            // {
            //     let samples_per_frame: u32 = (SAMPLE_RATE * 100) / (FPS * 100.0) as u32;
            //     let samples = &emu.mmu.apu.generate(samples_per_frame as usize);
            //     let mut audio_data = audio_buffer.lock().unwrap();

            //     for i in 0..samples.len() {
            //         audio_data[i as usize] = samples[i as usize];
            //     }
            // }

            // {
            //     let &(ref lock, ref cvar) = &*audio_sync_pair;
            //     let mut consumed = lock.lock().unwrap();
            //     consumed = cvar.wait(consumed).unwrap();
            //     while *consumed {
            //         consumed = cvar.wait(consumed).unwrap();
            //     }
            // }

            // FIXME: enable capture-at-frame again
            // if let Some(frm) = capture_at_frame {
            //     if frm == frame_counter {
            //         capture_frame(capture_filename, frame_counter, &emu.mmu.lcd).unwrap();
            //     }
            // }

            /*
            let mut texture = texture_creator
                .create_texture_streaming(fmt, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
                .unwrap();
                */

            texture
                .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                    buffer.copy_from_slice(&emu.mmu.lcd.buf_rgba8);
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
                    Rect::new(
                        0,
                        0, /*WINDOW_HEIGHT as i32*/
                        WINDOW_WIDTH,
                        WINDOW_HEIGHT,
                    ),
                )
                .unwrap();

            canvas.present();

            // FIXME: allow exit at specific frame
            // if let Some(frm) = exit_at_frame {
            //     if frm == frame_counter {
            //         println!("Exit at frame {}", frame_counter);
            //         break 'running;
            //     }
            // }

            emu.mmu.display_updated = false;
            frame_counter = frame_counter + 1;
        }
    }

    // FIXME: it should be paused...
    // audio_device.pause();
    return Ok(());
}
