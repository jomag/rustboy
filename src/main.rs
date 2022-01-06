extern crate clap;
extern crate ctrlc;
extern crate image;
extern crate num_traits;
extern crate png;
extern crate sdl2;

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
mod timer;
mod ui;

use emu::Emu;
use lcd::{LCD, SCREEN_HEIGHT, SCREEN_WIDTH};
use ui::audio::SAMPLE_RATE;
use ui::full::run_with_full_ui;
use ui::minimal::run_with_minimal_ui;

const APPNAME: &str = "Rustboy?";
const VERSION: &str = "0.0.0";
const AUTHOR: &str = "Jonatan Magnusson <jonatan.magnusson@gmail.com>";
const BOOTSTRAP_ROM: &str = "rom/boot.gb";
const CARTRIDGE_ROM: &str = "rom/tetris.gb";

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
    let _headless: bool = matches.is_present("headless");
    let _record: Option<&str> = matches.value_of("record");
    let _record_frame_skip: u32 = parse(matches.value_of("skip"), 3);
    let break_at_address: Option<u16> = parse_optional(matches.value_of("break"));
    let break_at_cycle: Option<u64> = parse_optional(matches.value_of("break-cycle"));
    let _break_at_frame: Option<u32> = parse_optional(matches.value_of("break-frame"));
    let _exit_at_cycle: Option<u32> = parse_optional(matches.value_of("exit-cycle"));
    let exit_at_frame: Option<u32> = parse_optional(matches.value_of("exit-frame"));
    let capture_at_frame: Option<u32> = parse_optional(matches.value_of("capture"));
    let capture_filename: &str = matches
        .value_of("capture-to")
        .unwrap_or("capture-frame-#.png");

    let mut emu = Emu::new(SAMPLE_RATE);
    emu.init();

    println!("Loading bootstrap ROM: {}", bootstrap_rom);
    let sz = emu.load_bootstrap(bootstrap_rom);
    println!(" - {} bytes read", sz);

    println!("Loading cartridge ROM: {}", cartridge_rom);
    emu.load_cartridge(cartridge_rom);

    run_with_full_ui(emu);
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
