use std::fmt::UpperHex;
use std::ops::Sub;

use crate::gameboy::debug::format_mnemonic;
use crate::gameboy::emu::Emu;
use crate::gameboy::instructions;
use crate::gameboy::registers::Registers;

// cycle   reg   prev reg   frm
// 0       5     0
// 0       5     0
// 0       5     0
// 0       5     0
// 1       20    5
// 1       20    5
// 1       20    5
// 1       20    5
// 2       12    12
// 2       12    12
// 2       12    12
// 2       12    12

use egui::{Context, Label, RichText, Ui};

pub struct RegistersView {
    prev: Registers,
    compare_with: Registers,
    prev_cycle: u64,
}

impl RegistersView {
    pub fn new() -> Self {
        return RegistersView {
            prev: Registers::new(),
            compare_with: Registers::new(),
            prev_cycle: 0,
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

    pub fn render(&mut self, ui: &mut Ui, emu: &Emu) {
        let reg = &emu.mmu.reg;

        ui.scope(|ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            ui.horizontal(|ui| {
                RegistersView::render_register(ui, "A:", reg.a, self.compare_with.a);
                RegistersView::render_register(ui, "B:", reg.b, self.compare_with.b);
                RegistersView::render_register(ui, "C:", reg.c, self.compare_with.c);
                RegistersView::render_register(ui, "D:", reg.d, self.compare_with.d);
                RegistersView::render_register(ui, "E:", reg.e, self.compare_with.e);
                RegistersView::render_register(ui, "F:", reg.get_f(), self.compare_with.get_f());
                RegistersView::render_register(ui, "H:", reg.h, self.compare_with.h);
                RegistersView::render_register(ui, "L:", reg.l, self.compare_with.l);
            });
            ui.horizontal(|ui| {
                RegistersView::render_register(ui, "SP:", reg.sp, self.compare_with.sp);
                RegistersView::render_register(ui, "PC:", reg.pc, self.compare_with.pc);
                ui.label(format!("Cycle: {}", emu.mmu.timer.abs_cycle));
            });
        });

        // The previous register values should only be updated when
        // another instructions has been executed. There's currently
        // no better way to do that than to check if PC has changed
        // since last render. This is only an approximation as the PC
        // can also be changed by the debugger, and the PC may not
        // change if it's on a jump instruction to the same address.
        if self.prev_cycle != emu.mmu.timer.abs_cycle {
            self.compare_with = self.prev;
            self.prev = emu.mmu.reg;
            self.prev_cycle = emu.mmu.timer.abs_cycle;
        }
    }
}

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
    fn stop_address(&mut self, emu: &Emu, lines: usize) -> usize {
        let mut adr = self.start_address;

        for _ in 0..lines {
            match instructions::op_length(emu.mmu.direct_read(adr)) {
                Some(len) => adr += len,
                None => break,
            }
        }

        adr
    }

    fn update_range(&mut self, emu: &Emu, lines: usize) {
        if !self.follow_pc {
            return;
        }

        let pc = emu.mmu.reg.pc as usize;

        if pc < self.start_address {
            self.start_address = pc;
            return;
        }

        let stop_address = self.stop_address(emu, lines);
        if pc > stop_address {
            self.start_address = pc;
            return;
        }
    }

    fn render_content(&mut self, ui: &mut Ui, emu: &Emu, lines: usize) {
        let mut addr = self.start_address;
        let pc = emu.mmu.reg.pc as usize;

        for _ in 0..lines {
            let text = format!("{:04x}: {}", addr, format_mnemonic(&emu.mmu, addr));

            let lbl;
            if addr == pc {
                let bg = ui.visuals().selection.bg_fill;
                let fg = ui.visuals().selection.stroke.color;
                lbl = Label::new(RichText::new(text).background_color(bg).color(fg));
            } else {
                lbl = Label::new(text);
            }

            ui.add(lbl);

            match instructions::op_length(emu.mmu.direct_read(addr)) {
                Some(len) => addr += len,
                None => break,
            }
        }
    }

    pub fn render(&mut self, ui: &mut Ui, emu: &Emu) {
        ui.scope(|ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            let row_height = 16.0; //ui.fonts().row_height(TextStyle::Monospace) + 2.0;
            let avail_height = ui.available_height();
            let lines = (avail_height / row_height) as usize;
            if lines >= 1 {
                self.update_range(emu, lines - 1);
                self.render_content(ui, emu, lines - 1);
            }
            ui.allocate_space(ui.available_size());
        });
    }
}

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

    pub fn render(&mut self, ctx: &Context, emu: &mut Emu, open: &mut bool) {
        egui::Window::new("Debugger")
            .open(open)
            .resizable(true)
            .show(ctx, |ui| {
                self.registers_view.render(ui, &emu);
                ui.separator();
                self.dis_view.render(ui, &emu);
            });
    }
}
