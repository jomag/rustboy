extern crate clap;
extern crate ctrlc;
extern crate num_traits;
extern crate png;
extern crate winit;

#[macro_use]
mod macros;

mod conv;
mod gameboy;
mod test_runner;
mod ui;
mod utils;
mod wave_audio_recorder;

use gameboy::emu::Emu;
use ui::full::*;

use gameboy::emu::Machine;

const APPNAME: &str = "Rustboy?";
const VERSION: &str = "0.0.0";
const AUTHOR: &str = "Jonatan Magnusson <jonatan.magnusson@gmail.com>";
const BOOTSTRAP_ROM: &str = "rom/boot.gb";
const CARTRIDGE_ROM: &str = "rom/tetris.gb";

const CLOCK_SPEED: usize = 4194304;
const CYCLES_PER_FRAME: usize = 70224;

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
            --break-frame=[N]       'Break at frame N'
            --exit-cycle=[N]        'Exit at cycle N'
            --ff-bootstrap          'Fast forward bootstrap'
            -R, --record=[PATH]     'Record into directory'
            -s, --skip=[N]          'Frames to skip while recording'
            -C, --capture=[N]       'Capture screen at frame N'
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
    let _break_at_frame: Option<u32> = parse_optional(matches.value_of("break-frame"));
    let _exit_at_cycle: Option<u32> = parse_optional(matches.value_of("exit-cycle"));
    let debug_log: Option<&str> = matches.value_of("debug-log");
    let ff_bootstrap = matches.is_present("ff-bootstrap");

    let machine = handle_machine_option(matches.value_of("machine"))?;

    let mut emu = Emu::new(machine);
    emu.init();

    println!("Loading bootstrap ROM: {}", bootstrap_rom);
    let sz = emu.load_bootstrap(bootstrap_rom);
    println!(" - {} bytes read", sz);

    println!("Loading cartridge ROM: {}", cartridge_rom);
    emu.load_cartridge(cartridge_rom);

    let mut debug = gameboy::debug::Debug::new();

    match debug_log {
        Some(filename) => debug.start_debug_log(filename),
        None => {}
    };

    if ff_bootstrap {
        println!("Fast forward bootstrap ...");
        while emu.mmu.bootstrap_mode {
            // println!(
            //     "@{:04x}, LY: 0x{:02x} ({})",
            //     emu.mmu.reg.pc, emu.mmu.ppu.ly, emu.mmu.ppu.ly
            // );
            emu.mmu.exec_op();
        }
        println!("Bootstrap mode disabled");
    }

    if let Some(expect) = test_expect {
        // This never returns
        test_runner::test_runner_expect(expect, &mut emu);
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
