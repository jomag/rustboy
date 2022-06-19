use egui::Context;
use egui_wgpu_backend::RenderPass;
use wgpu::{Device, Queue};

use crate::debug::Debug;
use crate::gameboy::emu::Emu;
use crate::gameboy::ppu::SCREEN_HEIGHT;
use crate::ui::serial_window::SerialWindow;
use crate::APPNAME;

use super::super::{breakpoints_window::BreakpointsWindow, render_stats::RenderStats};

use super::{
    audio_window::render_audio_window, cartridge_window::CartridgeWindow,
    debug_window::DebugWindow, memory_window::MemoryWindow, oam_window::render_oam_window,
    ppu_window::render_video_window, vram_window::VRAMWindow,
};

pub trait MainWindow<T> {
    fn init(&mut self, device: &Device, egui_rpass: &mut RenderPass);
    fn append_serial(&mut self, data: u8);

    fn render(
        &mut self,
        ctx: &Context,
        emu: &mut T,
        debug: &mut Debug,
        queue: &Queue,
        render_stats: &RenderStats,
    );
}

pub struct GameboyMainWindow {
    vram_window: VRAMWindow,
    vram_window_open: bool,

    debug_window: DebugWindow,
    debug_window_open: bool,

    breakpoints_window: BreakpointsWindow,
    breakpoints_window_open: bool,

    pub serial_window: SerialWindow,
    serial_window_open: bool,

    cartridge_window: CartridgeWindow,
    cartridge_window_open: bool,

    memory_window: MemoryWindow,
    memory_window_open: bool,

    audio_window_open: bool,
    ppu_window_open: bool,
    oam_window_open: bool,
}

impl MainWindow<Emu> for GameboyMainWindow {
    fn init(&mut self, device: &Device, rpass: &mut RenderPass) {
        self.vram_window.init(device, rpass);
    }

    fn append_serial(&mut self, data: u8) {
        self.serial_window.append(data)
    }

    fn render(
        &mut self,
        ctx: &Context,
        emu: &mut Emu,
        debug: &mut Debug,
        queue: &Queue,
        render_stats: &RenderStats,
    ) {
        self.render_toolbar(ctx, emu, debug);
        self.render_menu(ctx);

        self.vram_window
            .render(ctx, emu, queue, &mut self.vram_window_open);
        self.debug_window
            .render(ctx, emu, &mut self.debug_window_open);
        self.breakpoints_window
            .render(ctx, debug, &mut self.breakpoints_window_open);
        self.serial_window.render(ctx, &mut self.serial_window_open);
        self.cartridge_window
            .render(ctx, emu, &mut self.cartridge_window_open);
        self.memory_window
            .render(ctx, emu, &mut self.memory_window_open);

        render_audio_window(ctx, emu, &mut self.audio_window_open);
        render_video_window(ctx, emu, &mut self.ppu_window_open);
        render_oam_window(ctx, emu, &mut self.oam_window_open);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(APPNAME);
            ui.label(format!("UI FPS: {:.1}", render_stats.fps()));
            ui.label(format!("Emulator FPS: {:.10}", render_stats.fps()));
            egui::warn_if_debug_build(ui);
        });
    }
}

impl GameboyMainWindow {
    pub fn new() -> Self {
        GameboyMainWindow {
            vram_window: VRAMWindow::new(),
            vram_window_open: false,
            debug_window: DebugWindow::new(),
            debug_window_open: false,
            breakpoints_window: BreakpointsWindow::new(),
            breakpoints_window_open: false,
            serial_window: SerialWindow::new(),
            serial_window_open: false,
            cartridge_window: CartridgeWindow::new(),
            cartridge_window_open: false,
            memory_window: MemoryWindow::new(),
            memory_window_open: false,
            audio_window_open: false,
            ppu_window_open: false,
            oam_window_open: false,
        }
    }

    fn render_toolbar(&mut self, ctx: &Context, emu: &mut Emu, debug: &mut Debug) {
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
                if ui.button("Next scanline").clicked() {
                    debug.break_on_scanline((emu.mmu.ppu.ly + 1) % SCREEN_HEIGHT);
                    debug.continue_execution();
                }
                if ui.button("Reset").clicked() {
                    emu.reset();
                }
            });
        });
    }

    fn render_menu(&mut self, ctx: &Context) {
        egui::SidePanel::left("left_menu_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                if ui.selectable_label(self.vram_window_open, "VRAM").clicked() {
                    self.vram_window_open = !self.vram_window_open;
                }

                if ui
                    .selectable_label(self.serial_window_open, "Serial")
                    .clicked()
                {
                    self.serial_window_open = !self.serial_window_open;
                }

                if ui
                    .selectable_label(self.debug_window_open, "Debugger")
                    .clicked()
                {
                    self.debug_window_open = !self.debug_window_open;
                }

                if ui
                    .selectable_label(self.breakpoints_window_open, "Breakpoints")
                    .clicked()
                {
                    self.breakpoints_window_open = !self.breakpoints_window_open;
                }

                if ui
                    .selectable_label(self.cartridge_window_open, "Cartridge")
                    .clicked()
                {
                    self.cartridge_window_open = !self.cartridge_window_open;
                }

                if ui
                    .selectable_label(self.memory_window_open, "Memory")
                    .clicked()
                {
                    self.memory_window_open = !self.memory_window_open;
                }

                if ui
                    .selectable_label(self.audio_window_open, "APU (Audio)")
                    .clicked()
                {
                    self.audio_window_open = !self.audio_window_open;
                }

                if ui
                    .selectable_label(self.ppu_window_open, "PPU (Video)")
                    .clicked()
                {
                    self.ppu_window_open = !self.ppu_window_open;
                }

                if ui
                    .selectable_label(self.oam_window_open, "OAM (Sprites)")
                    .clicked()
                {
                    self.oam_window_open = !self.oam_window_open;
                }
            });
        });
    }
}
