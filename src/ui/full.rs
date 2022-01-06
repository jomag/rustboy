use crate::{
    emu::Emu,
    lcd::{SCREEN_HEIGHT, SCREEN_WIDTH},
    APPNAME,
};

// use eframe::{
//     egui::{self},
//     epi,
// };

use super::render_stats::RenderStats;

// use eframe::{egui, epi};

struct App {
    emu: Emu,
    fb_width: usize,
    fb_height: usize,
    fb_texture: Option<egui::TextureId>,
    render_stats: RenderStats,
}

impl App {
    pub fn new(emu: Emu) -> Self {
        App {
            emu,
            fb_width: SCREEN_WIDTH,
            fb_height: SCREEN_HEIGHT,
            fb_texture: None,
            render_stats: Default::default(),
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        APPNAME
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        // Update render stats with new frame info
        self.render_stats
            .on_new_frame(ctx.input().time, frame.info().cpu_usage);

        // Render texture from LCD framebuffer
        if self.fb_texture.is_none() || true {
            let image = epi::Image::from_rgba_unmultiplied(
                [self.fb_width, self.fb_height],
                &self.emu.mmu.lcd.buf_rgba8,
            );

            let texture = frame.alloc_texture(image);
            self.fb_texture = Some(texture);
        }

        if let Some(texture) = self.fb_texture {
            egui::Window::new("Gameboy").show(ctx, |ui| {
                let size = egui::Vec2::new(self.fb_width as f32, self.fb_height as f32);
                ui.image(texture, size);
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(APPNAME);
            ui.label(format!("FPS: {:.1}", self.render_stats.fps()));
        });

        frame.set_window_size(ctx.used_size());

        self.emu.mmu.display_updated = false;
        while !self.emu.mmu.display_updated {
            self.emu.mmu.exec_op();
        }

        frame.request_repaint();
    }
}

fn create_display(
    event_loop: &glutin::event_loop::EventLoop<()>,
) -> (
    glutin::WindowedContext<glutin::PossiblyCurrent>,
    glow::Context,
) {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("egui_glow example");

    let gl_window = unsafe {
        glutin::ContextBuilder::new()
            .with_depth_buffer(0)
            .with_srgb(true)
            .with_stencil_buffer(0)
            .with_vsync(true)
            .build_windowed(window_builder, event_loop)
            .unwrap()
            .make_current()
            .unwrap()
    };

    let gl = unsafe { glow::Context::from_loader_function(|s| gl_window.get_proc_address(s)) };

    unsafe {
        use glow::HasContext as _;
        gl.enable(glow::FRAMEBUFFER_SRGB);
    }

    (gl_window, gl)
}

pub fn run_with_full_ui<'a>(emu: Emu) {
    let mut clear_color = [0.1, 0.1, 0.3];

    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let (gl_window, gl) = create_display(&event_loop);
    let mut egui_glow = egui_glow::EguiGlow::new(&gl_window, &gl);

    let app = App::new(emu);

    event_loop.run(move |event, _, control_flow| {
        let (needs_repaint, shapes) = egui_glium.run(&display, |egui_ctx| {
            app.update();
        });
    });

    // eframe::run_native(Box::new(app), options);
}

pub fn run_with_full_eframe_ui<'a>(emu: Emu) {
    // let options = eframe::NativeOptions::default();
    // let app = App::new(emu);
    // eframe::run_native(Box::new(app), options);
}
