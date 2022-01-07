use crate::{
    emu::Emu,
    lcd::{SCREEN_HEIGHT, SCREEN_WIDTH},
    APPNAME,
};

use super::render_stats::RenderStats;

use epi::App as _;
use std::sync::Arc;

struct GlowRepaintSignal(std::sync::Mutex<glutin::event_loop::EventLoopProxy<RequestRepaintEvent>>);

impl epi::backend::RepaintSignal for GlowRepaintSignal {
    fn request_repaint(&self) {
        self.0.lock().unwrap().send_event(RequestRepaintEvent).ok();
    }
}

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
    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        // Update render stats with new frame info
        // self.render_stats
        //     .on_new_frame(ctx.input().time, frame.info().cpu_usage);

        // Render texture from LCD framebuffer
        if self.fb_texture.is_none() || true {
            let image = epi::Image::from_rgba_unmultiplied(
                [self.fb_width, self.fb_height],
                &self.emu.mmu.lcd.buf_rgba8,
            );

            // if let Some(texture_id) = tex_mngr.texture(frame, &response.url, image) {
            //     let mut size = egui::Vec2::new(image.size[0] as f32, image.size[1] as f32);
            //     size *= (ui.available_width() / size.x).min(1.0);
            //     ui.image(texture_id, size);
            // }

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

        self.emu.mmu.display_updated = false;
        while !self.emu.mmu.display_updated {
            self.emu.mmu.exec_op();
        }
    }

    fn name(&self) -> &str {
        APPNAME
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
    let clear_color = [0.1, 0.1, 0.3];

    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let (gl_window, gl) = create_display(&event_loop);
    let mut egui_glow = egui_glow::EguiGlow::new(&gl_window, &gl);

    let repaint_signal = std::sync::Arc::new(GlowRepaintSignal(std::sync::Mutex::new(
        event_loop.create_proxy(),
    )));

    let mut app = App::new(emu);

    let mut integration = egui_winit::epi::EpiIntegration::new(
        "egui_glow",
        gl_window.window(),
        repaint_signal,
        persistence,
        app,
    );

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            let (needs_repaint, shapes) = egui_glow.run(gl_window.window(), |ctx| {
                app.update(ctx, frame);
            });

            *control_flow = if needs_repaint {
                gl_window.window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            } else {
                glutin::event_loop::ControlFlow::Wait
            };

            {
                unsafe {
                    use glow::HasContext as _;
                    gl.clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }
                egui_glow.paint(&gl_window, &gl, shapes);
                gl_window.swap_buffers().unwrap();
            }
        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            glutin::event::Event::WindowEvent { event, .. } => {
                use glutin::event::WindowEvent;
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                if let glutin::event::WindowEvent::Resized(physical_size) = event {
                    gl_window.resize(physical_size);
                }

                egui_glow.on_event(&event);

                // TODO: ask egui if the events warrants a repaint instead
                gl_window.window().request_redraw();
            }

            glutin::event::Event::LoopDestroyed => {
                egui_glow.destroy(&gl);
            }

            _ => (),
        }
    });
}
