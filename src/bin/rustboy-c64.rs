use std::io::{self, ErrorKind};

use clap::Parser;
use rustboy::{
    c64::core::{CoreC64, Machine},
    ui::{app::MoeApp, c64::main_window_c64::MainWindowC64},
};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Kernal ROM
    #[clap(short, long, value_parser, default_value = "rom/c64/kernal.bin")]
    kernal: String,

    /// Char ROM
    #[clap(short, long, value_parser, default_value = "rom/c64/char.bin")]
    char: String,

    /// Basic ROM
    #[clap(short, long, value_parser, default_value = "rom/c64/basic.bin")]
    basic: String,
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

    let machine = Machine::C64;
    let mut core = CoreC64::new(machine);

    println!("Loading kernal: {}", args.kernal);
    match core.bus.load_kernal_rom(&args.kernal) {
        Ok(_) => {}
        Err(e) => match e.kind() {
            ErrorKind::NotFound => panic!("File not found"),
            e => panic!("Failed to load kernal: {:?}", e),
        },
    };

    println!("Loading character ROM: {}", args.char);
    match core.bus.load_char_rom(&args.char) {
        Ok(_) => {}
        Err(e) => match e.kind() {
            ErrorKind::NotFound => panic!("File not found"),
            e => panic!("Failed to load character ROM: {:?}", e),
        },
    };

    println!("Loading BASIC ROM: {}", args.basic);
    match core.bus.load_basic_rom(&args.basic) {
        Ok(_) => {}
        Err(e) => match e.kind() {
            ErrorKind::NotFound => panic!("File not found"),
            e => panic!("Failed to load BASIC ROM: {:?}", e),
        },
    };

    core.init();

    let mut debug = rustboy::debug::Debug::new();
    debug.break_execution();

    let main_window = MainWindowC64::new();
    let app = MoeApp::new(core, main_window);
    app.run_with_wgpu(debug);

    return Ok(());
}
