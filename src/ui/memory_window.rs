use egui::{CtxRef, ScrollArea, Ui};

use crate::emu::Emu;

pub struct MemoryView {
    mem_size: usize,
}

impl MemoryView {
    const bytes_per_row: usize = 16;

    pub fn new(mem_size: usize) -> Self {
        MemoryView { mem_size }
    }

    fn render_row(offset: usize, ui: &mut Ui, emu: &Emu) {
        let mut hex_str = String::with_capacity(MemoryView::bytes_per_row * 3);
        let mut char_str = String::with_capacity(MemoryView::bytes_per_row);

        for i in 0..=(MemoryView::bytes_per_row - 1) {
            let b = emu.mmu.direct_read(offset + i);
            hex_str.push_str(&format!(" {:02X}", b));
            char_str.push(match b {
                32..=126 => b as char,
                _ => '.',
            });
        }

        ui.label(format!("{:04X} {} {}", offset, hex_str, char_str));
    }

    pub fn render(&mut self, ctx: &CtxRef, ui: &mut Ui, emu: &Emu) {
        ui.scope(|ui| {
            let text_style = egui::TextStyle::Monospace;
            let row_height = ui.fonts()[text_style].row_height();
            let num_rows = self.mem_size / MemoryView::bytes_per_row;

            ui.style_mut().override_text_style = Some(text_style);

            ScrollArea::vertical().auto_shrink([false; 2]).show_rows(
                ui,
                row_height,
                num_rows,
                |ui, row_range| {
                    for row in row_range {
                        MemoryView::render_row(row * MemoryView::bytes_per_row, ui, emu);
                    }
                },
            )
        });
    }
}

pub struct MemoryWindow {
    mem_view: MemoryView,
}

impl MemoryWindow {
    pub fn new() -> Self {
        MemoryWindow {
            mem_view: MemoryView::new(0x10000),
        }
    }

    pub fn render(&mut self, ctx: &CtxRef, emu: &mut Emu) {
        egui::Window::new("Memory").resizable(true).show(ctx, |ui| {
            ui.label("TEXT");
            ui.separator();
            self.mem_view.render(ctx, ui, emu);
        });
    }
}
