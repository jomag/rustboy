use std::{
    collections::HashMap, fs::File, io::BufWriter, iter, sync::Arc, thread::sleep, time::Instant,
};

use crate::{
    apu::apu::AudioRecorder,
    buttons::ButtonType,
    debug::Debug,
    emu::Emu,
    lcd::{SCREEN_HEIGHT, SCREEN_WIDTH},
    wave_audio_recorder::WaveAudioRecorder,
    APPNAME, CLOCK_SPEED, CYCLES_PER_FRAME,
};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, SampleFormat, Stream, StreamConfig,
};
use egui::{FontDefinitions, Key};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use epi::*;
use ringbuf::{Consumer, RingBuffer};
use wgpu::{Device, FilterMode, Queue, Surface, SurfaceConfiguration};
use winit::{
    event::{Event::*, StartCause},
    event_loop::ControlFlow,
    window::Window,
};

use super::{
    audio_window::render_audio_window, breakpoints_window::BreakpointsWindow,
    cartridge_window::CartridgeWindow, debug_window::DebugWindow, memory_window::MemoryWindow,
    render_stats::RenderStats, serial_window::SerialWindow,
};

const TARGET_FPS: u64 = 30;

/// A custom event type for the winit app.
enum AppEvent {
    RequestRedraw,
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
struct ExampleRepaintSignal(std::sync::Mutex<winit::event_loop::EventLoopProxy<AppEvent>>);

impl epi::backend::RepaintSignal for ExampleRepaintSignal {
    fn request_repaint(&self) {
        self.0
            .lock()
            .unwrap()
            .send_event(AppEvent::RequestRedraw)
            .ok();
    }
}

struct MoeApp {
    emu: Emu,
    fb_width: usize,
    fb_height: usize,
    fb_texture: Option<egui::TextureId>,
    serial_buffer_consumer: Option<Consumer<u8>>,

    // Statistics for the UI frame rate
    ui_render_stats: RenderStats,

    // Statistics for the emulator frame rate
    emu_render_stats: RenderStats,
    previous_frame_time: Option<f32>,

    // Windows
    debug_window: DebugWindow,
    breakpoints_window: BreakpointsWindow,
    serial_window: SerialWindow,
    cartridge_window: CartridgeWindow,
    memory_window: MemoryWindow,

    keymap: HashMap<Key, ButtonType>,
}

impl MoeApp {
    pub fn setup_serial(&mut self) {
        let buf = RingBuffer::<u8>::new(128);
        let (producer, consumer) = buf.split();
        self.emu.mmu.serial.output = Some(producer);
        self.serial_buffer_consumer = Some(consumer);
    }

    fn run_until_next_frame(&mut self, debug: &mut Debug) {
        self.emu.mmu.display_updated = false;
        while debug.before_op(&self.emu) && !self.emu.mmu.display_updated {
            self.emu.mmu.exec_op();
        }
    }

    fn render_next_frame(
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

        // Copy Gameboy screen to texture if it has changed since last render
        if self.fb_texture.is_none() || self.emu.mmu.display_updated {
            let texture_size = wgpu::Extent3d {
                width: SCREEN_WIDTH as u32,
                height: SCREEN_HEIGHT as u32,
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

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &self.emu.mmu.lcd.buf_rgba8,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * SCREEN_WIDTH as u32),
                    rows_per_image: std::num::NonZeroU32::new(SCREEN_HEIGHT as u32),
                },
                texture_size,
            );

            let texture_id =
                egui_rpass.egui_texture_from_wgpu_texture(&device, &texture, FilterMode::Nearest);

            self.fb_texture = Some(texture_id);
        }

        // Build the whole app UI
        self.update(&platform.context(), &mut frame, debug);

        // End the UI frame
        let (output, paint_commands) = platform.end_frame(Some(&window));
        let paint_jobs = platform.context().tessellate(paint_commands);

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

        egui_rpass.update_texture(&device, &queue, &platform.context().font_image());
        egui_rpass.update_user_textures(&device, &queue);
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

        if output.needs_repaint {
            window.request_redraw();
        }
    }

    pub fn setup_audio(&mut self) -> Stream {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let recorder = WaveAudioRecorder {
            mono_writer: Some(hound::WavWriter::create("mono.wav", spec).unwrap()),
            gen1_writer: Some(hound::WavWriter::create("gen1.wav", spec).unwrap()),
            gen2_writer: Some(hound::WavWriter::create("gen2.wav", spec).unwrap()),
        };

        // self.emu.mmu.apu.recorder = Some(Box::new(recorder));

        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");

        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");

        let config = supported_configs_range
            .next()
            .expect("no supported config?")
            .with_max_sample_rate();

        println!("Selected audio config: {:?}", config);

        // Generate ringbuffer big enough to fit 4 frames of audio.
        // A new sample is generated every fourth clock cycle.
        // FIXME: the buffer is way too big as it is, so that there is some time
        // before it runs out of space. This is because the data is not pulled in
        // the right speed.
        let buf = RingBuffer::<f32>::new((CYCLES_PER_FRAME as usize / 4) * 4 * 100);
        let (producer, mut consumer) = buf.split();
        self.emu.mmu.apu.buf = Some(producer);

        let err_fn = |err| eprintln!("an error occured on the output audio stream: {}", err);
        let sample_format = config.sample_format();
        let config: StreamConfig = config.into();

        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;

        let step = ((CYCLES_PER_FRAME as f64) / ((sample_rate as f64) / (TARGET_FPS as f64))) / 4.0;

        let mut avg: u8 = 0;

        let mut next_value = move || {
            // println!("enter next_value");
            // avg = (avg + 1) % 3;
            // if avg == 0 {
            //     consumer.discard(23);
            // } else {
            //     consumer.discard(22);
            // }

            consumer.discard(step as usize);

            // println!("remaining samples: {}", consumer.remaining());
            match consumer.pop() {
                Some(sample) => {
                    // println!("Sample: {}", sample);
                    sample
                }
                None => {
                    // println!("Oops! Out of audio data");
                    0.0
                }
            }
        };

        let stream = match sample_format {
            SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // println!("HERE");
                    write_beep::<f32>(data, channels, &mut next_value)
                },
                err_fn,
            ),

            SampleFormat::I16 => device.build_output_stream(
                &config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    // println!("HERE2");
                    write_beep::<i16>(data, channels, &mut next_value)
                },
                err_fn,
            ),

            SampleFormat::U16 => device.build_output_stream(
                &config,
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    // println!("HERE3");
                    write_beep::<u16>(data, channels, &mut next_value)
                },
                err_fn,
            ),
        }
        .unwrap();

        fn write_beep<T: Sample>(
            output: &mut [T],
            channels: usize,
            next_sample: &mut dyn FnMut() -> f32,
        ) {
            // println!("BEEP?!");
            for frame in output.chunks_mut(channels) {
                let value: T = cpal::Sample::from::<f32>(&next_sample());
                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        }

        stream.play().unwrap();
        stream
    }

    pub fn new(emu: Emu) -> Self {
        MoeApp {
            emu,
            fb_width: SCREEN_WIDTH,
            fb_height: SCREEN_HEIGHT,
            fb_texture: None,
            ui_render_stats: Default::default(),
            emu_render_stats: Default::default(),
            debug_window: DebugWindow::new(),
            breakpoints_window: BreakpointsWindow::new(),
            serial_window: SerialWindow::new(),
            cartridge_window: CartridgeWindow::new(),
            memory_window: MemoryWindow::new(),
            serial_buffer_consumer: None,
            keymap: HashMap::from([
                (Key::ArrowLeft, ButtonType::Left),
                (Key::ArrowRight, ButtonType::Right),
                (Key::ArrowUp, ButtonType::Up),
                (Key::ArrowDown, ButtonType::Down),
                (Key::Z, ButtonType::A),
                (Key::X, ButtonType::B),
                (Key::Enter, ButtonType::Start),
                (Key::Space, ButtonType::Select),
            ]),
            previous_frame_time: None,
        }
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame, debug: &mut Debug) {
        if let Some(ref mut consumer) = self.serial_buffer_consumer {
            while let Some(ch) = consumer.pop() {
                self.serial_window.append(ch);
            }
        }

        // Handle keyboard input
        if ctx.wants_keyboard_input() {
            self.emu.mmu.buttons.release_all();
        } else {
            let inp = ctx.input();
            for key in self.keymap.keys() {
                if inp.key_down(*key) {
                    self.emu.mmu.buttons.handle_press(self.keymap[&key])
                }
                if inp.key_released(*key) {
                    self.emu.mmu.buttons.handle_release(self.keymap[&key])
                }
            }
        }

        // Update render stats with new frame info
        self.ui_render_stats
            .on_new_frame(ctx.input().time, frame.info().cpu_usage);

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

        render_audio_window(ctx, &mut self.emu);
        self.debug_window.render(ctx, &mut self.emu, debug);
        self.breakpoints_window.render(ctx, &mut self.emu, debug);
        self.serial_window.render(ctx);
        self.cartridge_window.render(ctx, &mut self.emu);
        self.memory_window.render(ctx, &mut self.emu);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(APPNAME);
            ui.label(format!("UI FPS: {:.1}", self.ui_render_stats.fps()));
            ui.label(format!("Emulator FPS: {:.10}", self.emu_render_stats.fps()));
            egui::warn_if_debug_build(ui);
        });
    }
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

pub fn run_with_wgpu(emu: Emu, mut debug: Debug) {
    let mut app = MoeApp::new(emu);
    let event_loop = winit::event_loop::EventLoop::with_user_event();
    let window = winit::window::WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_title(APPNAME)
        .with_inner_size(winit::dpi::PhysicalSize {
            width: 2800,
            height: 1800,
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

    let _stream = app.setup_audio();
    app.setup_serial();

    event_loop.run(move |event, _, control_flow| {
        if false {
            print_event(&event);
        }

        // Pass the winit events to the platform integration.
        platform.handle_event(&event);

        match event {
            RedrawRequested(..) => {
                platform.update_time(start_time.elapsed().as_secs_f64());
                app.render_next_frame(
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
                const US_PER_FRAME: u64 = 1_000_000 / TARGET_FPS;
                let one_frame_duration = std::time::Duration::from_micros(US_PER_FRAME);
                let now = Instant::now();

                // let elapsed_time = now.duration_since(emulator_frame_timestamp).as_micros() as u64;

                if now >= next_frame_instant {
                    // Run emulator until next frame is ready
                    app.run_until_next_frame(&mut debug);

                    // Flush recorded audio every frame
                    if let Some(ref mut rec) = app.emu.mmu.apu.recorder {
                        rec.flush()
                    }

                    // Calculate the time for the next frame to be rendered
                    next_frame_instant = next_frame_instant + one_frame_duration;

                    // Special handling is time is out of sync so that the
                    // next frame should already be rendered.
                    if now > next_frame_instant {
                        next_frame_instant = now;
                    }

                    // Record frame render time statistics
                    let abs_elapsed_time = now.duration_since(start_time).as_secs_f64();
                    app.emu_render_stats
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
