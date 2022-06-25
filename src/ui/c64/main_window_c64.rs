use egui::Context;

use crate::debug::Debug;
use crate::ui::memory_window::MemoryWindow;
use crate::{c64::core::CoreC64, ui::main_window::MainWindow, APPNAME};

use super::debug_window::DebugWindow;

pub struct MainWindowC64 {
    debug_window: DebugWindow,
    debug_window_open: bool,

    memory_window: MemoryWindow,
    memory_window_open: bool,
}

impl MainWindow<CoreC64> for MainWindowC64 {
    fn init(&mut self, _device: &wgpu::Device, _egui_rpasss: &mut egui_wgpu_backend::RenderPass) {}

    fn append_serial(&mut self, _data: u8) {
        todo!()
    }

    fn render(
        &mut self,
        ctx: &egui::Context,
        core: &mut CoreC64,
        debug: &mut crate::debug::Debug,
        _queue: &wgpu::Queue,
        render_stats: &crate::ui::render_stats::RenderStats,
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

impl MainWindowC64 {
    pub fn new() -> Self {
        MainWindowC64 {
            debug_window: DebugWindow::new(),
            debug_window_open: true,
            memory_window: MemoryWindow::new(),
            memory_window_open: true,
        }
    }

    fn render_toolbar(&mut self, ctx: &Context, _core: &mut CoreC64, debug: &mut Debug) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Break").clicked() {
                    debug.break_execution();
                    // self.debug_window_open = true;
                };
                if ui.button("Step").clicked() {
                    debug.step();
                };
                if ui.button("Continue").clicked() {
                    debug.continue_execution();
                };
                if ui.button("Next scanline").clicked() {
                    // debug.break_on_scanline((core.mmu.ppu.ly + 1) % core.screen_height());
                    // debug.continue_execution();
                    todo!();
                }
                if ui.button("Reset").clicked() {
                    // core.reset();
                    todo!();
                }
            });
        });
    }
}
