use egui::{
    emath, epaint, pos2, style, vec2, Color32, CtxRef, Pos2, Rect, Sense, Shape, Stroke, Ui,
};

use crate::{apu::CH3_WAVE_LENGTH, emu::Emu};

pub fn render_wavetable(ui: &mut Ui, emu: &mut Emu) {
    let height = ui.spacing().slider_width;
    let size = vec2(ui.available_size_before_wrap().x, height);
    let (rect, _) = ui.allocate_at_least(size, Sense::hover());
    let style = ui.style().noninteractive();

    let mut shapes = Vec::with_capacity(3 + CH3_WAVE_LENGTH * 2);
    shapes.push(Shape::Rect(epaint::RectShape {
        rect,
        corner_radius: style.corner_radius,
        fill: ui.visuals().extreme_bg_color,
        stroke: ui.style().noninteractive().bg_stroke,
    }));

    let color = ui.visuals().text_color();

    let wavetable_rect = Rect::from_x_y_ranges(0.0..=CH3_WAVE_LENGTH as f32, 15.0..=0.0);
    let to_screen = emath::RectTransform::from_to(wavetable_rect, rect);
    let line_stroke = Stroke::new(1.0, color);

    for n in 0..CH3_WAVE_LENGTH {
        let p1 = pos2(n as f32, emu.mmu.apu.ch3.wave[n] as f32);
        let p1 = to_screen.transform_pos_clamped(p1);

        let p2 = pos2((n + 1) as f32, emu.mmu.apu.ch3.wave[n] as f32);
        let p2 = to_screen.transform_pos_clamped(p2);

        shapes.push(Shape::line_segment([p1, p2], line_stroke));

        if n < CH3_WAVE_LENGTH - 1 {
            let p3 = pos2((n + 1) as f32, emu.mmu.apu.ch3.wave[n + 1] as f32);
            let p3 = to_screen.transform_pos_clamped(p3);
            shapes.push(Shape::line_segment([p2, p3], line_stroke));
        }
    }

    ui.painter().extend(shapes);
}

pub fn render_audio_window(ctx: &CtxRef, emu: &mut Emu) {
    egui::Window::new("Audio").show(ctx, |ui| {
        ui.heading("Channel 1");
        ui.label(format!("Enabled: {}", emu.mmu.apu.s1.enabled));
        ui.label(format!("Envelope: {}", emu.mmu.apu.s1.envelope));

        ui.heading("Channel 2");
        ui.label(format!("Enabled: {}", emu.mmu.apu.s2.enabled));
        ui.label(format!("Envelope: {}", emu.mmu.apu.s2.envelope));

        ui.heading("Channel 3");
        ui.label(format!("Enabled: {}", emu.mmu.apu.ch3.enabled));
        ui.label(format!("Volume Code: {}", emu.mmu.apu.ch3.volume_code));
        ui.label(format!(
            "Length counter: {}",
            emu.mmu.apu.ch3.length_counter.value,
        ));
        ui.label(format!(
            "Frequency timer: {}",
            emu.mmu.apu.ch3.frequency_timer
        ));
        ui.label(format!("Wave position: {}", emu.mmu.apu.ch3.wave_position));
        render_wavetable(ui, emu);

        ui.heading("Channel 4");
        ui.label(format!("Enabled: {}", emu.mmu.apu.ch4.enabled));
        ui.label(format!("LFSR: {}", emu.mmu.apu.ch4.lfsr));
        ui.label(format!(
            "Frequency timer: {}",
            emu.mmu.apu.ch4.frequency_timer
        ));
    });
}
