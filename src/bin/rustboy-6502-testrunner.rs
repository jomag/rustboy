use std::{
    fs::File,
    io::{self, ErrorKind, Read},
};

use clap::Parser;
use egui::Context;

use rustboy::ui::memory_window::MemoryWindow;
use rustboy::APPNAME;
use rustboy::{debug::Breakpoint, ui::c64::debug_window::DebugWindow};
use rustboy::{debug::Debug, ui::main_window::MainWindow};

use rustboy::{
    core::Core,
    cpu::cpu_6510::{disassemble_one, op_len, CPU, OPS},
    ui::app::MoeApp,
    MemoryMapped,
};

struct Bus6502 {
    pub mem: Box<[u8]>,
}

impl MemoryMapped for Bus6502 {
    fn read(&self, adr: usize) -> u8 {
        self.mem[adr]
    }

    fn write(&mut self, adr: usize, value: u8) {
        self.mem[adr] = value;
    }

    fn reset(&mut self) {}
}

impl Bus6502 {
    pub fn new() -> Self {
        Bus6502 {
            mem: vec![0; 0x10000].into_boxed_slice(),
        }
    }

    pub fn load(&mut self, path: &str) -> Result<(), io::Error> {
        let mut file = File::open(path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        self.mem = content.into_boxed_slice();
        return Ok(());
    }

    pub fn load_buf(&mut self, buf: Vec<u8>, offset: usize) {
        self.mem[offset..(offset + buf.len())].copy_from_slice(&buf);
    }
}

struct Core6502 {
    pub bus: Bus6502,
    pub cpu: CPU,
    frame: usize,
}

impl Core for Core6502 {
    fn screen_width(&self) -> usize {
        16
    }

    fn screen_height(&self) -> usize {
        16
    }

    fn handle_press(&self) {}
    fn handle_release(&self) {}
    fn release_all(&mut self) {}

    fn current_frame(&self) -> usize {
        self.frame / 1024
    }

    fn log_state(&self, _: &mut std::fs::File) {}

    fn op_offset(&self) -> usize {
        self.cpu.op_offset().into()
    }

    fn scanline(&self) -> usize {
        0
    }

    fn at_source_code_breakpoint(&self) -> bool {
        false
    }

    fn exec_op(&mut self) {
        self.cpu.exec(&mut self.bus);
        self.frame += 1;
    }

    fn update_input_state(&mut self, _state: &egui::InputState) {}
    fn register_serial_output_buffer(&mut self, _p: ringbuf::Producer<u8>) {}
    fn set_audio_rates(&mut self, _clock_rate: f64, _sample_rate: f64) {}
    fn end_audio_frame(&mut self) {}
    fn push_audio_samples(&mut self, _p: &mut ringbuf::Producer<i16>) {}
    fn to_rgba8(&self, _dst: &mut Box<[u8]>, _palette: Vec<(u8, u8, u8)>) {}

    fn op_length(&self, adr: usize) -> usize {
        let code = self.bus.read(adr);
        let op = &OPS[usize::from(code)];
        op_len(&op.adr) as usize
    }

    fn format_op(&self, adr: usize) -> (String, usize) {
        let mut next: usize = 0;
        let text = disassemble_one(&self.bus, adr, &mut next);
        (text, next)
    }

    fn read(&self, adr: usize) -> u8 {
        self.bus.read(adr)
    }

    fn write(&mut self, adr: usize, value: u8) {
        self.bus.write(adr, value);
    }

    fn reset(&mut self) {
        self.cpu.reset(&self.bus)
    }
}

impl Core6502 {
    pub fn new() -> Self {
        Core6502 {
            cpu: CPU::new(),
            bus: Bus6502::new(),
            frame: 0,
        }
    }

    pub fn init(&mut self) {
        self.cpu.reset(&self.bus);
    }

    pub fn print_state(&self, data: u8) {
        let c = self.cpu;
        let (dis, _) = self.format_op(c.op_offset().into());
        println!(
            "clk:{} a:{:02x} x:{:02x} y:{:02x} p:{:02x} sp:{:02x} pc:{:04x} ir:{:02x} sync:{}|@{:04X}:\"{}\" (op cycle {})",
            c.cycles,
            c.a,
            c.x,
            c.y,
            c.p,
            c.sp,
            c.pc,
            c.get_ir(),
            if c.sync { 1 } else { 0 },
            c.op_offset(),
            dis,
            c.op_cycle,
        );
    }

    pub fn one_cycle(&mut self) {
        // let data = match self.cpu.wr {
        //     true => {
        //         self.bus.write(self.cpu.adr.into(), self.cpu.data);
        //         self.cpu.data
        //     }
        //     false => self.bus.read(self.cpu.adr.into()),
        // };

        // let data = self.bus.read(self.cpu.adr.into());
        // println!(
        //     "Preparing data for memory address 0x{:04x}: {:02x}",
        //     self.cpu.adr, data
        // );

        self.cpu.one_cycle(&mut self.bus);
    }
}

pub struct MainWindow6502 {
    debug_window: DebugWindow,
    debug_window_open: bool,
    memory_window: MemoryWindow,
    memory_window_open: bool,
}

impl MainWindow<Core6502> for MainWindow6502 {
    fn init(&mut self, _device: &wgpu::Device, _egui_rpasss: &mut egui_wgpu_backend::RenderPass) {}
    fn append_serial(&mut self, _data: u8) {}

    fn render(
        &mut self,
        ctx: &egui::Context,
        core: &mut Core6502,
        debug: &mut rustboy::debug::Debug,
        _queue: &wgpu::Queue,
        render_stats: &rustboy::ui::render_stats::RenderStats,
    ) {
        self.render_toolbar(ctx, core, debug);

        self.debug_window
            .render(ctx, &core.cpu, core, &mut self.debug_window_open);
        self.memory_window
            .render(ctx, core, &mut self.memory_window_open);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(APPNAME);
            ui.label(format!("UI FPS: {:.1}", render_stats.fps()));
            ui.label(format!("Emulator FPS: {:.10}", render_stats.fps()));
            egui::warn_if_debug_build(ui);
        });
    }
}

impl MainWindow6502 {
    pub fn new() -> Self {
        MainWindow6502 {
            debug_window: DebugWindow::new(),
            debug_window_open: true,
            memory_window: MemoryWindow::new(),
            memory_window_open: true,
        }
    }

    fn render_toolbar(&mut self, ctx: &Context, core: &mut Core6502, debug: &mut Debug) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Break").clicked() {
                    debug.break_execution();
                    self.debug_window_open = true;
                };
                if ui.button("Step").clicked() {
                    debug.step();
                };
                if ui.button("Continue").clicked() {
                    debug.continue_execution();
                };
                if ui.button("Reset").clicked() {
                    core.reset();
                }
            });
        });
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Test binary
    #[clap(short, long, value_parser)]
    bin: Option<String>,

    /// Raw hexcode input
    #[clap(short, long, value_parser)]
    raw: Option<String>,

    /// Start address
    #[clap(short, long, value_parser, default_value = "1024")]
    start: usize,

    /// With UI and debugger
    #[clap(short, long, value_parser)]
    ui: bool,

    /// Step one cycle at a time instead of a full operation
    #[clap(long, value_parser)]
    single_cycle: bool,
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

    let mut core = Core6502::new();

    if args.bin.is_none() && args.raw.is_none() {
        println!("One of --bin and --raw must be specified");
        return Ok(());
    }

    match args.bin {
        Some(bin) => {
            println!("Loading binary: {}", bin);
            match core.bus.load(&bin) {
                Ok(_) => {}
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => panic!("File not found"),
                    e => panic!("Failed to load kernal: {:?}", e),
                },
            };
        }
        None => {}
    }

    match args.raw {
        Some(raw) => {
            let raw = raw.replace(" ", "");
            let decoded = hex::decode(raw).expect("Decoding raw input failed");
            core.bus.load_buf(decoded, args.start);
        }
        None => {}
    }

    core.init();
    core.cpu.pc = args.start as u16;

    let mut debug = rustboy::debug::Debug::new();

    if args.ui {
        debug.break_execution();
        debug.add_breakpoint(0x37ed, Breakpoint { enabled: true });
        // debug.add_breakpoint(0x596, Breakpoint { enabled: true });
        let main_window = MainWindow6502::new();
        let app = MoeApp::new(core, main_window);
        app.run_with_wgpu(debug);
    } else {
        let mut progress: usize = 0;
        let mut stuck_count: usize = 0;
        let mut prev_pc = core.op_offset();

        // core.print_state(core.bus.read(core.cpu.adr.into()));
        // println!("");

        while debug.before_op(&mut core) && core.cpu.cycles < 400000 {
            core.print_state(core.bus.read(core.cpu.adr.into()));

            if args.single_cycle {
                core.one_cycle();
            } else {
                core.exec_op();
            }

            progress += 1;
            if progress % 10000 == 0 {
                println!("Cycle {}", progress);
            }

            let pc = core.op_offset();

            if pc != prev_pc {
                stuck_count = 0;
            } else {
                stuck_count += 1;
                if stuck_count == 50 {
                    println!("PC seems to be stuck at {:04x}", pc);
                    break;
                }
            }

            prev_pc = pc;
        }
    }

    return Ok(());
}
