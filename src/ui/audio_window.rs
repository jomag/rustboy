use egui::{emath, epaint, pos2, vec2, Context, Rect, Sense, Shape, Stroke, Ui};

use crate::{apu::wave_gen::CH3_WAVE_MEMORY_SIZE, emu::Emu};

pub fn render_wavetable(ui: &mut Ui, emu: &mut Emu) {
    let sample_count = CH3_WAVE_MEMORY_SIZE * 2;

    let height = ui.spacing().slider_width;
    let size = vec2(ui.available_size_before_wrap().x, height);
    let (rect, _) = ui.allocate_at_least(size, Sense::hover());
    let style = ui.style().noninteractive();

    let mut shapes = Vec::with_capacity(3 + CH3_WAVE_MEMORY_SIZE * 2);
    shapes.push(Shape::Rect(epaint::RectShape {
        rect,
        rounding: style.rounding,
        fill: ui.visuals().extreme_bg_color,
        stroke: ui.style().noninteractive().bg_stroke,
    }));

    let color = ui.visuals().text_color();

    let wavetable_rect = Rect::from_x_y_ranges(0.0..=sample_count as f32, 15.0..=0.0);
    let to_screen = emath::RectTransform::from_to(wavetable_rect, rect);
    let line_stroke = Stroke::new(1.0, color);

    let mut prev = pos2(0.0, 0.0);

    for n in 0..sample_count {
        let sample = emu.mmu.apu.ch3.get_sample(n) as f32;

        let p1 = pos2(n as f32, sample);
        let p1 = to_screen.transform_pos_clamped(p1);

        let p2 = pos2(n as f32 + 1.0, sample);
        let p2 = to_screen.transform_pos_clamped(p2);

        if n > 0 {
            shapes.push(Shape::line_segment([prev, p1], line_stroke));
        }

        shapes.push(Shape::line_segment([p1, p2], line_stroke));
        prev = p2;
    }

    ui.painter().extend(shapes);
}

pub fn render_audio_window(ctx: &Context, emu: &mut Emu, open: &mut bool) {
    egui::Window::new("Audio").open(open).show(ctx, |ui| {
        ui.heading("Channel 1");
        ui.label(format!("Enabled: {}", emu.mmu.apu.s1.enabled));
        ui.label(format!("Envelope: {}", emu.mmu.apu.s1.envelope));
        ui.label(format!("Frequency: {}", emu.mmu.apu.s1.frequency));

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
