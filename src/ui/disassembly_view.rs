use egui::{Label, RichText, Ui};

use crate::core::Core;

pub struct DisassemblyView {
    start_address: usize,
    follow_pc: bool,
}

impl DisassemblyView {
    pub fn new() -> Self {
        DisassemblyView {
            start_address: 0,
            follow_pc: true,
        }
    }

    // Find the last visible address
    fn stop_address(&mut self, core: &impl Core, lines: usize) -> usize {
        let mut adr = self.start_address;

        for _ in 0..lines {
            adr += core.op_length(adr);
        }

        adr
    }

    fn update_range(&mut self, core: &impl Core, lines: usize) {
        if !self.follow_pc {
            return;
        }

        let pc = core.op_offset();

        if pc < self.start_address {
            self.start_address = pc;
            return;
        }

        let stop_address = self.stop_address(core, lines);
        if pc > stop_address {
            self.start_address = pc;
            return;
        }
    }

    fn render_content(&mut self, ui: &mut Ui, core: &impl Core, lines: usize) {
        let mut adr = self.start_address;
        let pc = core.op_offset();

        for _ in 0..lines {
            let (text, next) = core.format_op(adr);
            let text = format!("{:04x}: {}", adr, text);

            let lbl;
            if adr == pc {
                let bg = ui.visuals().selection.bg_fill;
                let fg = ui.visuals().selection.stroke.color;
                lbl = Label::new(RichText::new(text).background_color(bg).color(fg));
            } else {
                lbl = Label::new(text);
            }

            ui.add(lbl);
            adr = next;
        }
    }

    pub fn render(&mut self, ui: &mut Ui, core: &impl Core) {
        ui.scope(|ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            let row_height = 16.0; //ui.fonts().row_height(TextStyle::Monospace) + 2.0;
            let avail_height = ui.available_height();
            let lines = (avail_height / row_height) as usize;
            if lines >= 1 {
                self.update_range(core, lines - 1);
                self.render_content(ui, core, lines - 1);
            }
            ui.allocate_space(ui.available_size());
        });
    }
}
