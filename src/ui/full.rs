use std::time::Instant;

use crate::{
    emu::Emu,
    lcd::{SCREEN_HEIGHT, SCREEN_WIDTH},
    APPNAME,
};

use super::render_stats::RenderStats;

use epi::App;
use epi::NativeTexture;
use glium::glutin;
use glium::Display;
use glutin::event::StartCause;

const TARGET_FPS: u64 = 60;

struct MoeApp {
    emu: Emu,
    fb_width: usize,
    fb_height: usize,
    fb_texture: Option<egui::TextureId>,
    render_stats: RenderStats,
}

impl MoeApp {
    pub fn new(emu: Emu) -> Self {
        MoeApp {
            emu,
            fb_width: SCREEN_WIDTH,
            fb_height: SCREEN_HEIGHT,
            fb_texture: None,
            render_stats: Default::default(),
        }
    }
}

impl MoeApp {
    fn update(&mut self, ctx: &egui::CtxRef /*, frame: &epi::Frame*/) {
        // Update render stats with new frame info
        // self.render_stats
        // .on_new_frame(ctx.input().time, frame.info().cpu_usage);

        if let Some(texture_id) = self.fb_texture {
            egui::Window::new("Gameboy").show(ctx, |ui| {
                let scale: usize = 3;
                let size = egui::Vec2::new(
                    (self.fb_width * scale) as f32,
                    (self.fb_height * scale) as f32,
                );
                ui.image(texture_id, size);
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(APPNAME);
            ui.label(format!("FPS: {:.1}", self.render_stats.fps()));
        });
    }
}

fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("egui_glium example");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop).unwrap()
}

// fn create_glow_display(
//     event_loop: &glutin::event_loop::EventLoop<()>,
// ) -> (
//     glutin::WindowedContext<glutin::PossiblyCurrent>,
//     glow::Context,
// ) {
//     let window_builder = glutin::window::WindowBuilder::new()
//         .with_resizable(true)
//         .with_inner_size(glutin::dpi::LogicalSize {
//             width: 800.0,
//             height: 600.0,
//         })
//         .with_title("egui_glow example");

//     let gl_window = unsafe {
//         glutin::ContextBuilder::new()
//             .with_depth_buffer(0)
//             .with_srgb(true)
//             .with_stencil_buffer(0)
//             .with_vsync(true)
//             .build_windowed(window_builder, event_loop)
//             .unwrap()
//             .make_current()
//             .unwrap()
//     };

//     let gl = unsafe { glow::Context::from_loader_function(|s| gl_window.get_proc_address(s)) };

//     unsafe {
//         use glow::HasContext as _;
//         gl.enable(glow::FRAMEBUFFER_SRGB);
//     }

//     (gl_window, gl)
// }

pub fn run_with_pure_glium_ui<'a>(emu: Emu) {
    let mut app = MoeApp::new(emu);

    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let display = create_display(&event_loop);

    let mut egui_glium = egui_glium::EguiGlium::new(&display);

    event_loop.run(move |event, _, control_flow| {
        let start_time = Instant::now();

        let mut redraw = |next_frame: bool| {
            let mut quit = false;

            // Render texture from LCD framebuffer
            if app.fb_texture.is_none() || true {
                let image_size = (app.fb_width as u32, app.fb_height as u32);
                let image = glium::texture::RawImage2d::from_raw_rgba(
                    app.emu.mmu.lcd.buf_rgba8.to_vec(),
                    image_size,
                );

                // Load to gpu memory
                let glium_texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();
                // let glow_texture = glow::texture::SrgbTexture2d::new(&display, image).unwrap();

                // Allow us to share the texture with egui:
                let glium_texture = std::rc::Rc::new(glium_texture);
                // let glow_texture = std::rc::Rc::new(glow_texture);

                // Allocate egui's texture id for GL texture
                let texture_id = egui_glium.painter.register_native_texture(glium_texture);
                // let texture_id = egui_glium.painter.register_native_texture(glow_texture);
                app.fb_texture = Some(texture_id);
            }

            let (needs_repaint, shapes) = egui_glium.run(&display, |egui_ctx| {
                // let frame: epi::Frame::new();
                app.update(egui_ctx); //, &frame);
            });

            if next_frame {
                app.emu.mmu.display_updated = false;
                while !app.emu.mmu.display_updated {
                    app.emu.mmu.exec_op();
                }
                // frame.request_repaint();
            }

            *control_flow = if quit {
                glutin::event_loop::ControlFlow::Exit
            } else if needs_repaint {
                display.gl_window().window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            } else {
                glutin::event_loop::ControlFlow::Wait
            };

            {
                use glium::Surface as _;
                let mut target = display.draw();

                let color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                target.clear_color(color[0], color[1], color[2], color[3]);

                // draw things behind egui here

                egui_glium.paint(&display, &mut target, shapes);

                // draw things on top of egui here

                target.finish().unwrap();
            }
        };

        // println!("EVENT {:?}", event);

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(false),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(false),

            glutin::event::Event::NewEvents(StartCause::ResumeTimeReached { .. }) => redraw(true),

            glutin::event::Event::WindowEvent { event, .. } => {
                use glutin::event::WindowEvent;

                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                egui_glium.on_event(&event);
                display.gl_window().window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }

            _ => (),
        }

        let elapsed_time = Instant::now().duration_since(start_time).as_millis() as u64;
        let wait_millis = match 1000 / TARGET_FPS >= elapsed_time {
            true => 1000 / TARGET_FPS - elapsed_time,
            false => 0,
        };
        let new_inst = start_time + std::time::Duration::from_millis(wait_millis);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(new_inst);
    });
}
