use egui::{Button, Color32, Context, TextEdit};

use crate::debug::{Breakpoint, Debug};
use crate::emu::Emu;

pub struct BreakpointsWindow {
    add_breakpoint_input: String,
}

impl BreakpointsWindow {
    pub fn new() -> Self {
        BreakpointsWindow {
            add_breakpoint_input: "".to_string(),
        }
    }

    pub fn render(&mut self, ctx: &Context, emu: &mut Emu, debug: &mut Debug, open: &mut bool) {
        egui::Window::new("Breakpoints")
            .open(open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.scope(|ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                    ui.horizontal(
                        |ui| match u16::from_str_radix(&self.add_breakpoint_input, 16) {
                            Ok(adr) => {
                                ui.text_edit_singleline(&mut self.add_breakpoint_input);
                                if ui.button("✚").clicked() {
                                    debug.add_breakpoint(adr, Breakpoint { enabled: true });
                                }
                            }
                            Err(_) => {
                                ui.text_edit_singleline(&mut self.add_breakpoint_input);
                                ui.add_enabled(false, Button::new("✚"));
                            }
                        },
                    );

                    ui.separator();

                    egui::Grid::new("breakpoints_grid_id").show(ui, |ui| {
                        for (adr, ref mut bps) in debug.breakpoints.iter_mut() {
                            for bp in bps.iter_mut() {
                                let mut en = bp.enabled;
                                ui.checkbox(&mut en, "");
                                bp.enabled = en;
                                ui.label(format!("{:04X}", adr));
                                ui.end_row();
                            }
                        }
                    });

                    ui.allocate_space(ui.available_size());
                });
            });
    }
}
