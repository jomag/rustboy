use std::{iter, sync::Arc, time::Instant, usize::MAX};

use crate::{debug::Debug, gameboy::emu::Emu, APPNAME};
use egui::{FontDefinitions, Label};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use ringbuf::{Consumer, RingBuffer};
use wgpu::{Device, FilterMode, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;
use winit::{event::Event::*, event_loop::ControlFlow};

use crate::{core::Core, gameboy::CLOCK_SPEED};

use super::{
    audio_player::AudioPlayer, gameboy::main_window::MainWindow, render_stats::RenderStats,
};

pub const PIXEL_SIZE: usize = 4;
pub const TARGET_FPS: f64 = 59.727500569606;

/// A custom event type for the winit app.
pub enum AppEvent {
    RequestRedraw,
}

pub enum System {
    Gameboy(Emu),
    C64(Emu),
}

/// Debug function to print event details
fn print_event(event: &winit::event::Event<AppEvent>) {
    match &event {
        NewEvents(start_cause) => match start_cause {
            // StartCause::ResumeTimeReached { .. } => assert!(false, "RESUME TIME REACHED!"),
            _ => println!("\n--> NewEvents: {:?}", start_cause),
        },
        RedrawRequested(..) => println!("--> RedrawRequested"),
        WindowEvent { event, .. } => println!("--> WindowEvent {:?}", event),
        DeviceEvent { event, .. } => println!("--> DeviceEvent {:?}", event),
        UserEvent(..) => println!("--> UserEvent"),
        Suspended => println!("--> Suspended"),
        Resumed => println!("--> Resumed"),
        MainEventsCleared => println!("--> MainEventsCleared"),
        RedrawEventsCleared => println!("--> RedrawEventsCleared"),
        LoopDestroyed => println!("--> LoopDestroyed"),
    }
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
pub struct ExampleRepaintSignal(std::sync::Mutex<winit::event_loop::EventLoopProxy<AppEvent>>);

impl epi::backend::RepaintSignal for ExampleRepaintSignal {
    fn request_repaint(&self) {
        self.0
            .lock()
            .unwrap()
            .send_event(AppEvent::RequestRedraw)
            .ok();
    }
}

pub struct MoeApp<T: Core, W: MainWindow<T>> {
    fb_width: usize,
    fb_height: usize,
    fb_texture: Option<egui::TextureId>,
    fb_texture_frame: usize,

    serial_buffer_consumer: Option<Consumer<u8>>,
    audio: AudioPlayer,
    texture_buffer: Box<[u8]>,

    // Statistics for the UI frame rate
    ui_render_stats: RenderStats,

    // Statistics for the emulator frame rate
    pub emu_render_stats: RenderStats,
    previous_frame_time: Option<f32>,

    core: T,
    main_window: W,
}

impl<T: 'static + Core, W: 'static + MainWindow<T>> MoeApp<T, W> {
    pub fn setup_serial(&mut self) {
        let buf = RingBuffer::<u8>::new(128);
        let (producer, consumer) = buf.split();
        self.core.register_serial_output_buffer(producer);
        self.serial_buffer_consumer = Some(consumer);
    }

    pub fn setup_audio(&mut self) {
        self.audio.setup();
        self.core.set_audio_rates(CLOCK_SPEED as f64 / 4.0, 44100.0)
    }

    pub fn run_until_next_frame(&mut self, debug: &mut Debug) {
        let frame = self.core.current_frame();

        while debug.before_op(&mut self.core) && frame == self.core.current_frame() {
            self.core.exec_op();
        }

        if self.core.current_frame() != frame {
            self.core.end_audio_frame();
            match self.audio.producer {
                Some(ref mut p) => self.core.push_audio_samples(p),
                None => {}
            }
        }
    }

    fn render_texture(&mut self) {
        let palette: [(u8, u8, u8); 4] = [
            (0x9B, 0xBC, 0x0F),
            (0x8B, 0xAC, 0x0F),
            (0x30, 0x62, 0x30),
            (0x0f, 0x38, 0x0f),
        ];

        self.core
            .to_rgba8(&mut self.texture_buffer, palette.to_vec())
    }

    pub fn render_next_frame(
        &mut self,
        platform: &mut Platform,
        surface: &Surface,
        window: &Window,
        repaint_signal: &Arc<ExampleRepaintSignal>,
        device: &Device,
        queue: &Queue,
        egui_rpass: &mut RenderPass,
        surface_config: &SurfaceConfiguration,
        debug: &mut Debug,
    ) {
        let output_frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
                // This error occurs when the app is minimized on Windows.
                // Silently return here to prevent spamming the console with:
                // "The underlying surface has changed, and therefore the swap chain must be updated"
                return;
            }
            Err(e) => {
                eprintln!("Dropped frame with error: {}", e);
                return;
            }
        };

        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Begin to draw the UI frame.
        let egui_start = Instant::now();
        platform.begin_frame();
        let app_output = epi::backend::AppOutput::default();

        let mut frame = epi::Frame::new(epi::backend::FrameData {
            info: epi::IntegrationInfo {
                name: "egui_example",
                web_info: None,
                cpu_usage: self.previous_frame_time,
                native_pixels_per_point: Some(window.scale_factor() as _),
                prefer_dark_mode: None,
            },
            output: app_output,
            repaint_signal: repaint_signal.clone(),
        });

        let (width, height, current_frame) = (
            self.core.screen_width() as u32,
            self.core.screen_height() as u32,
            self.core.current_frame(),
        );

        // Copy Gameboy screen to texture if it has changed since last render
        if self.fb_texture.is_none() || self.fb_texture_frame != current_frame {
            let texture_size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("emulator screen texture"),
            });

            let texture_view = texture.create_view(&Default::default());

            self.render_texture();

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &self.texture_buffer,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * width),
                    rows_per_image: std::num::NonZeroU32::new(height),
                },
                texture_size,
            );

            let texture_id = egui_rpass.egui_texture_from_wgpu_texture(
                &device,
                &texture_view,
                FilterMode::Nearest,
            );

            self.fb_texture = Some(texture_id);
            self.fb_texture_frame = current_frame;
        }

        // Build the whole app UI
        self.update(&platform.context(), &mut frame, debug, queue);

        // End the UI frame
        let frame_output = platform.end_frame(Some(&window));
        let paint_jobs = platform.context().tessellate(frame_output.shapes);

        let frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
        self.previous_frame_time = Some(frame_time);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder"),
        });

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: surface_config.width,
            physical_height: surface_config.height,
            scale_factor: window.scale_factor() as f32,
        };

        egui_rpass
            .add_textures(&device, &queue, &frame_output.textures_delta)
            .unwrap();

        // egui_rpass.update_texture(&device, &queue, &platform.context().font_image());
        // egui_rpass.update_user_textures(&device, &queue);

        egui_rpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        egui_rpass
            .execute(
                &mut encoder,
                &output_view,
                &paint_jobs,
                &screen_descriptor,
                Some(wgpu::Color::BLACK),
            )
            .unwrap();

        // Submit the commands.
        queue.submit(iter::once(encoder.finish()));

        // Redraw egui
        output_frame.present();

        if frame_output.needs_repaint {
            window.request_redraw();
        }
    }

    pub fn new(core: T, main_window: W) -> Self {
        let w = core.screen_width();
        let h = core.screen_height();

        MoeApp {
            audio: AudioPlayer::new(),
            fb_width: w,
            fb_height: h,
            fb_texture: None,
            fb_texture_frame: MAX,
            texture_buffer: vec![0; w * h * PIXEL_SIZE].into_boxed_slice(),
            ui_render_stats: Default::default(),
            emu_render_stats: Default::default(),
            serial_buffer_consumer: None,
            previous_frame_time: None,
            main_window,
            core,
        }
    }

    fn update(
        &mut self,
        ctx: &egui::Context,
        frame: &epi::Frame,
        debug: &mut Debug,
        queue: &Queue,
    ) {
        if let Some(ref mut consumer) = self.serial_buffer_consumer {
            while let Some(ch) = consumer.pop() {
                self.main_window.append_serial(ch)
            }
        }

        // Handle keyboard input
        if ctx.wants_keyboard_input() {
            self.core.release_all();
        } else {
            self.core.update_input_state(&ctx.input());
        }

        // Update render stats with new frame info
        self.ui_render_stats
            .on_new_frame(ctx.input().time, frame.info().cpu_usage);

        self.main_window
            .render(ctx, &mut self.core, debug, queue, &self.ui_render_stats);

        if let Some(texture_id) = self.fb_texture {
            egui::Window::new("Gameboy").show(ctx, |ui| {
                let scale: usize = 3;
                let size = egui::Vec2::new(
                    (self.fb_width * scale) as f32,
                    (self.fb_height * scale) as f32,
                );

                let r = ui.image(texture_id, size);
                match r.hover_pos() {
                    Some(p) => {
                        let x = (p[0] - r.rect.left()) as usize / scale;
                        let y = (p[1] - r.rect.top()) as usize / scale;
                        r.on_hover_ui_at_pointer(|ui| {
                            ui.add(Label::new(format!("({}, {})", x, y)));
                        });
                    }
                    None => {}
                }
            });
        }
    }

    pub fn run_with_wgpu(mut self, mut debug: Debug) {
        let event_loop = winit::event_loop::EventLoop::with_user_event();
        let window = winit::window::WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .with_title(APPNAME)
            .with_inner_size(winit::dpi::PhysicalSize {
                width: 2800 as u32,
                height: 1800 as u32,
            })
            .build(&event_loop)
            .unwrap();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };

        // WGPU 0.11+ support force fallback (if HW implementation not supported), set it to true or false (optional).
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))
        .unwrap();

        let size = window.inner_size();
        let surface_format = surface.get_preferred_format(&adapter).unwrap();
        let mut surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width as u32,
            height: size.height as u32,

            // Fifo - with vsync
            // Immediate - without vsync
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &surface_config);

        let repaint_signal = std::sync::Arc::new(ExampleRepaintSignal(std::sync::Mutex::new(
            event_loop.create_proxy(),
        )));

        // We use the egui_winit_platform crate as the platform.
        let mut platform = Platform::new(PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        // We use the egui_wgpu_backend crate as the render backend.
        let mut egui_rpass = RenderPass::new(&device, surface_format, 1);

        let start_time = Instant::now();

        // Time for when the next frame should be rendered
        let mut next_frame_instant = Instant::now();

        self.setup_audio();
        self.setup_serial();
        self.main_window.init(&device, &mut egui_rpass);

        event_loop.run(move |event, _, control_flow| {
            if false {
                print_event(&event);
            }

            // Pass the winit events to the platform integration.
            platform.handle_event(&event);

            match event {
                RedrawRequested(..) => {
                    platform.update_time(start_time.elapsed().as_secs_f64());
                    self.render_next_frame(
                        &mut platform,
                        &surface,
                        &window,
                        &repaint_signal,
                        &device,
                        &queue,
                        &mut egui_rpass,
                        &surface_config,
                        &mut debug,
                    );
                }

                MainEventsCleared => {
                    let one_frame_duration = std::time::Duration::from_secs_f64(1.0 / TARGET_FPS);
                    let now = Instant::now();

                    // let elapsed_time = now.duration_since(emulator_frame_timestamp).as_micros() as u64;

                    if now >= next_frame_instant {
                        // Run emulator until next frame is ready
                        self.run_until_next_frame(&mut debug);

                        // Calculate the time for the next frame to be rendered
                        next_frame_instant = next_frame_instant + one_frame_duration;

                        // Special handling is time is out of sync so that the
                        // next frame should already be rendered.
                        if now > next_frame_instant {
                            next_frame_instant = now;
                        }

                        // Record frame render time statistics
                        let abs_elapsed_time = now.duration_since(start_time).as_secs_f64();
                        self.emu_render_stats
                            .on_new_frame(abs_elapsed_time, Some(0.0));

                        window.request_redraw();
                    }

                    *control_flow = ControlFlow::WaitUntil(next_frame_instant);
                }

                WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::Resized(size) => {
                        // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                        // See: https://github.com/rust-windowing/winit/issues/208
                        // This solves an issue where the app would panic when minimizing on Windows.
                        if size.width > 0 && size.height > 0 {
                            surface_config.width = size.width;
                            surface_config.height = size.height;
                            surface.configure(&device, &surface_config);
                        }
                        window.request_redraw();
                    }

                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }

                    _ => {
                        // window.request_redraw();
                    }
                },

                _ => (),
            }
        });
    }
}
