
// extern crate sdl2;

// use emu::sdl2::event::Event;
// use emu::sdl2::keyboard::Keycode;

use sdl2;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::video::Window;
use sdl2::render::Texture;
use sdl2::render::Canvas;

use mmu::MMU;

struct SDLEnvironment<'a> {
    window: Window,
    canvas: Canvas<Window>,
    texture: Texture<'a>
}

pub struct EmuSDL<'a> {
    pub mmu: MMU,
    sdl_environment: Option<SDLEnvironment<'a>>
}

impl<'a> EmuSDL<'a> {
    pub fn new() -> Self {
        EmuSDL {
            mmu: MMU::new(),
            sdl_environment: None
        }
    }

    pub fn init(&mut self) {
        self.mmu.init();

        // Initialize UI
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("rustboy", 320 + 4, 288 + 4)
            // .position_centered()
            .position(100, 100)
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        let texture_creator = canvas.texture_creator();
        let fmt = PixelFormatEnum::RGB24;
        let mut texture = texture_creator.create_texture_streaming(fmt, 160, 144).unwrap();

        canvas.clear();
        canvas.copy(&texture, None, Some(Rect::new(2, 2, 320, 288))).unwrap();
        canvas.present();

        let mut event_pump = sdl_context.event_pump().unwrap();

        self.sdl_environment = Some(SDLEnvironment {
            window: window,
            canvas: canvas,
            texture: texture
        });
    }

    pub fn load_bootstrap(&mut self, path: &str) {
        self.mmu.load_bootstrap(&path);
    }

    pub fn load_cartridge(&mut self, path: &str) {
        self.mmu.load_cartridge(&path);
    }

    pub fn step(&mut self) {
        self.mmu.exec_op();

        if self.mmu.display_updated {
            let mut env = self.sdl_environment; //.unwrap();
            // let mut canvas = self.sdl_environment.unwrap().canvas;
            // let mut texture = self.sdl_environment.unwrap().texture;
            self.mmu.lcd.copy_to_texture(&mut env.texture);
            env.canvas.clear();
            env.canvas.copy(&env.texture, None, Some(Rect::new(2, 2, 320, 288))).unwrap();
            env.canvas.present();
            self.mmu.display_updated = false;
        }
    }
}