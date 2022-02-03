extern crate clap;
extern crate ctrlc;
extern crate num_traits;
extern crate png;
extern crate sdl2;
extern crate winit;

#[macro_use]
mod macros;

mod apu;
mod buttons;
mod cartridge;
mod debug;
mod dma;
mod emu;
mod instructions;
mod interrupt;
mod lcd;
mod mmu;
mod registers;
mod serial;
mod test_runner;
mod timer;
mod ui;
mod utils;

use emu::Emu;
use lcd::{LCD, SCREEN_HEIGHT, SCREEN_WIDTH};
use ui::full::*;

use crate::emu::Machine;

const APPNAME: &str = "Rustboy?";
const VERSION: &str = "0.0.0";
const AUTHOR: &str = "Jonatan Magnusson <jonatan.magnusson@gmail.com>";
const BOOTSTRAP_ROM: &str = "rom/boot.gb";
const CARTRIDGE_ROM: &str = "rom/tetris.gb";

const CLOCK_SPEED: u32 = 4194304;
const CYCLES_PER_FRAME: u32 = 70224;

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
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&lcd.buf_rgba8).unwrap();

    println!("Captured frame {}", frame);
    return Ok(());
}

fn handle_machine_option(opt: Option<&str>) -> Result<Machine, ()> {
    match opt {
        None => Ok(Machine::GameBoyDMG),
        Some("dmg") => Ok(Machine::GameBoyDMG),
        Some("cgb") => Ok(Machine::GameBoyCGB),
        Some(other) => {
            println!("Unsupported machine type: {}", other);
            println!("Supported types: dmg, cgb");
            Err(())
        }
    }
}

fn main() -> Result<(), ()> {
    let matches = clap::App::new(APPNAME)
        .version(VERSION)
        .author(AUTHOR)
        .about("Your less than average GameBoy emulator")
        .args_from_usage(
            "<ROM>                  'The ROM to run'
            -B, --boot=[FILE]       'Path to bootstrap ROM'
            -b, --break=[ADDR]      'Break at address ADDR'
            --break-cycle=[N]       'Break at cycle N'
            --break-frame=[N]       'Break at frame N'
            --exit-cycle=[N]        'Exit at cycle N'
            --exit-frame=[N]        'Exit at frame N'
            --ff-bootstrap          'Fast forward bootstrap'
            -R, --record=[PATH]     'Record into directory'
            -s, --skip=[N]          'Frames to skip while recording'
            -C, --capture=[N]       'Capture screen at frame N'
            --capture-to=[FILE]     'Capture filename'
            -t, --test=[VARIANT]    'Run test mode'
            --test-expect=[STR]     'Run test and validate serial output'
            --debug-log=[FILE]      'Write extensive debug info before each op'
            -m, --machine=[MACHINE] 'The machine to emulate'
            ",
        )
        .get_matches();

    let bootstrap_rom = matches.value_of("boot").unwrap_or(BOOTSTRAP_ROM); // done!
    let cartridge_rom = matches.value_of("ROM").unwrap_or(CARTRIDGE_ROM); // done!
    let test_runner_variant = matches.value_of("test");
    let test_expect = matches.value_of("test-expect");
    let _record: Option<&str> = matches.value_of("record");
    let _record_frame_skip: u32 = parse(matches.value_of("skip"), 3);
    let break_at_address: Option<u16> = parse_optional(matches.value_of("break"));
    let break_at_cycle: Option<u64> = parse_optional(matches.value_of("break-cycle"));
    let _break_at_frame: Option<u32> = parse_optional(matches.value_of("break-frame"));
    let _exit_at_cycle: Option<u32> = parse_optional(matches.value_of("exit-cycle"));
    let exit_at_frame: Option<u32> = parse_optional(matches.value_of("exit-frame"));
    let capture_at_frame: Option<u32> = parse_optional(matches.value_of("capture"));
    let debug_log: Option<&str> = matches.value_of("debug-log");
    let ff_bootstrap = matches.is_present("ff-bootstrap");

    let machine = handle_machine_option(matches.value_of("machine"))?;

    let capture_filename: &str = matches
        .value_of("capture-to")
        .unwrap_or("capture-frame-#.png");

    let mut emu = Emu::new(machine);
    emu.init();

    println!("Loading bootstrap ROM: {}", bootstrap_rom);
    let sz = emu.load_bootstrap(bootstrap_rom);
    println!(" - {} bytes read", sz);

    println!("Loading cartridge ROM: {}", cartridge_rom);
    emu.load_cartridge(cartridge_rom);

    let mut debug = crate::debug::Debug::new();

    match debug_log {
        Some(filename) => debug.start_debug_log(filename),
        None => {}
    };

    if ff_bootstrap {
        println!("Fast forward bootstrap ...");
        while emu.mmu.bootstrap_mode {
            emu.mmu.exec_op();
        }
        println!("Bootstrap mode disabled");
    }

    if let Some(expect) = test_expect {
        // This never returns
        test_runner::test_runner_expect(expect, &mut emu, &mut debug);
    }

    if let Some(variant) = test_runner_variant {
        // This never returns
        test_runner::test_runner(variant, &mut emu, &mut debug);
    }

    run_with_wgpu(emu, debug);
    // run_with_minimal_ui(APPNAME, None, None, &mut emu);

    // FIXME: add breakpoint from command line argument
    // if let Some(addr) = break_at_address {
    //     breakpoints.push(addr);
    // }

    // FIXME: add break at absolute cycle
    // if let Some(cycle) = break_at_cycle {
    //     emu.mmu.timer.abs_cycle_breakpoint = cycle;
    // }

    println!("Clean shutdown. Bye!");
    return Ok(());
}
