use egui::Context;
use egui_wgpu_backend::RenderPass;
use wgpu::{Device, Queue};

use super::render_stats::RenderStats;
use crate::debug::Debug;

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
