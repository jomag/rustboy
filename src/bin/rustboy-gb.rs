extern crate ctrlc;
extern crate num_traits;
extern crate png;
extern crate winit;

use clap::Parser;
use rustboy::gameboy::emu::Emu;
use rustboy::gameboy::emu::Machine;
use rustboy::gameboy::{BOOTSTRAP_ROM, CARTRIDGE_ROM};
use rustboy::ui::app::MoeApp;
use rustboy::ui::full::*;
use rustboy::ui::gameboy::main_window::GameboyMainWindow;

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

fn handle_machine_option(opt: Option<String>) -> Result<Machine, ()> {
    match opt.as_deref() {
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

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Cartridge ROM
    #[clap(name = "ROM", value_parser)]
    cartridge_rom: Option<String>,

    /// Boot ROM
    #[clap(short = 'B', long = "boot", value_parser)]
    boot_rom: Option<String>,

    /// Break at frame N
    #[clap(long, value_parser)]
    break_frame: Option<usize>,

    /// Exit at cycle N
    #[clap(long, value_parser)]
    exit_at_cycle: Option<usize>,

    /// Fast-forward boot sequence
    #[clap(long, action)]
    ff_bootstrap: bool,

    /// Record into this directory
    #[clap(short = 'R', long = "record", value_parser)]
    record_dir: Option<String>,

    /// Frames to skip while recording
    #[clap(short = 's', long, value_parser)]
    skip: Option<usize>,

    /// Capture screen content at frame N
    #[clap(short = 'C', long, value_parser)]
    capture: Option<usize>,

    /// Run in testing mode
    #[clap(short = 't', long = "test", value_parser)]
    test_variant: Option<String>,

    /// Expected serial output in test mode
    #[clap(long, value_parser)]
    test_expect: Option<String>,

    /// File to write debug log to
    #[clap(long, value_parser)]
    debug_log: Option<String>,

    // Machine type
    #[clap(short, long, value_parser)]
    machine: Option<String>,
}

fn main() -> Result<(), ()> {
    let args = Args::parse();

    let bootstrap_rom = args.boot_rom.unwrap_or(BOOTSTRAP_ROM.to_string());
    let cartridge_rom = args.cartridge_rom.unwrap_or(CARTRIDGE_ROM.to_string());
    let machine = handle_machine_option(args.machine)?;

    let mut emu = Emu::new(machine);
    emu.init();

    println!("Loading bootstrap ROM: {}", bootstrap_rom);
    let sz = emu.load_bootstrap(&bootstrap_rom.to_string());
    println!(" - {} bytes read", sz);

    println!("Loading cartridge ROM: {}", cartridge_rom.to_string());
    emu.load_cartridge(&cartridge_rom.to_string());

    let mut debug = rustboy::debug::Debug::new();

    match args.debug_log {
        Some(filename) => debug.start_debug_log(&filename),
        None => {}
    };

    if args.ff_bootstrap {
        println!("Fast forward bootstrap ...");
        while emu.mmu.bootstrap_mode {
            emu.mmu.exec_op();
        }
        println!("Bootstrap mode disabled");
    }

    if let Some(expect) = args.test_expect {
        // This never returns
        rustboy::test_runner::test_runner_expect(&expect, &mut emu);
    }

    if let Some(variant) = args.test_variant {
        // This never returns
        rustboy::test_runner::test_runner(&variant, &mut emu, &mut debug);
    }

    let main_window = GameboyMainWindow::new();
    let app = MoeApp::new(emu, main_window);
    app.run_with_wgpu(debug);

    println!("Clean shutdown. Bye!");
    return Ok(());
}
