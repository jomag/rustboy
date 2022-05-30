use egui::CtxRef;
use egui_wgpu_backend::RenderPass;
use wgpu::{Device, FilterMode, Queue};

use crate::{emu::Emu, ppu::PPU};

use super::tile_data_view::TileDataView;

pub struct VRAMWindow {
    tile_data_view: TileDataView,
}

impl VRAMWindow {
    pub fn new() -> Self {
        VRAMWindow {
            tile_data_view: TileDataView::new(),
        }
    }

    pub fn render(
        &mut self,
        ctx: &CtxRef,
        emu: &mut Emu,
        device: &Device,
        queue: &Queue,
        egui_rpass: &mut RenderPass,
    ) {
        egui::Window::new("Video RAM").show(ctx, |ui| {
            self.tile_data_view
                .render(ui, emu, device, queue, egui_rpass);
        });
    }
}
