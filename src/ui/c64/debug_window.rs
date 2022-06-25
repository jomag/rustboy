use std::{fmt::UpperHex, ops::Sub};

use egui::{Context, Label, RichText, Ui};

use crate::{core::Core, cpu::cpu_6510::CPU, ui::disassembly_view::DisassemblyView};

pub struct DebugWindow {
    dis_view: DisassemblyView,
    registers_view: RegistersView,
}

impl DebugWindow {
    pub fn new() -> Self {
        DebugWindow {
            dis_view: DisassemblyView::new(),
            registers_view: RegistersView::new(),
        }
    }

    pub fn render(&mut self, ctx: &Context, cpu: &CPU, core: &impl Core, open: &mut bool) {
        egui::Window::new("Debugger")
            .open(open)
            .resizable(true)
            .show(ctx, |ui| {
                self.registers_view.render(ui, cpu);
                ui.separator();
                self.dis_view.render(ui, core);
            });
    }
}

// Note that this is a complete copy from the Gameboy equivalent for now.
pub struct RegistersView {
    _prev: CPU,
    compare_with: CPU,
    _prev_cycle: u64,
}

impl RegistersView {
    pub fn new() -> Self {
        return RegistersView {
            _prev: CPU::new(),
            compare_with: CPU::new(),
            _prev_cycle: 0,
        };
    }

    fn render_register<T: UpperHex + PartialEq + Sub>(ui: &mut Ui, label: &str, value: T, prev: T) {
        ui.label(label);

        let value_text = match std::mem::size_of::<T>() {
            1 => format!("{:02X}", value),
            2 => format!("{:04X}", value),
            4 => format!("{:08X}", value),
            8 => format!("{:16X}", value),
            _ => format!("{:X}", value),
        };

        if prev != value {
            let bg = ui.visuals().selection.bg_fill;
            let fg = ui.visuals().selection.stroke.color;
            let lbl = Label::new(RichText::new(value_text).background_color(bg).color(fg));

            let prev_text = match std::mem::size_of::<T>() {
                1 => format!("Was: {:02X}", prev),
                2 => format!("Was: {:04X}", prev),
                4 => format!("Was: {:08X}", prev),
                8 => format!("Was: {:16X}", prev),
                _ => format!("Was: {:X}", prev),
            };

            ui.add(lbl).on_hover_text(prev_text);
        } else {
            ui.label(value_text);
        }
    }

    fn render_flags(ui: &mut Ui, p: u8, _prev: u8) {
        let mut render_flag = |name: &str, v: bool| {
            let fg = if v {
                ui.visuals().strong_text_color()
            } else {
                ui.visuals().text_color()
            };

            let lbl = Label::new(RichText::new(name).color(fg));
            ui.add(lbl);
        };

        render_flag("N", p & 0x80 != 0);
        render_flag("V", p & 0x40 != 0);
        render_flag("D", p & 0x08 != 0);
        render_flag("I", p & 0x04 != 0);
        render_flag("Z", p & 0x02 != 0);
        render_flag("C", p & 0x01 != 0);
    }

    pub fn render(&mut self, ui: &mut Ui, cpu: &CPU) {
        ui.scope(|ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            ui.horizontal(|ui| {
                RegistersView::render_register(ui, "A:", cpu.a, self.compare_with.a);
                RegistersView::render_register(ui, "X:", cpu.x, self.compare_with.x);
                RegistersView::render_register(ui, "Y:", cpu.y, self.compare_with.y);
                RegistersView::render_register(ui, "IR:", cpu.ir, self.compare_with.ir);
                RegistersView::render_register(ui, "P:", cpu.p, self.compare_with.p);
            });
            ui.horizontal(|ui| {
                RegistersView::render_register(ui, "SP:", cpu.sp, self.compare_with.sp);
                RegistersView::render_register(ui, "PC:", cpu.pc, self.compare_with.pc);
                RegistersView::render_flags(ui, cpu.p, self.compare_with.p);
            });
        });

        // The previous register values should only be updated when
        // another instructions has been executed. There's currently
        // no better way to do that than to check if PC has changed
        // since last render. This is only an approximation as the PC
        // can also be changed by the debugger, and the PC may not
        // change if it's on a jump instruction to the same address.

        // if self.prev_cycle != emu.mmu.timer.abs_cycle {
        //     self.compare_with = self.prev;
        //     self.prev = emu.mmu.reg;
        //     self.prev_cycle = emu.mmu.timer.abs_cycle;
        // }

        // FIXME: Important that the above is enabled again!
    }
}
