
extern crate sdl2;

use std::time::Duration;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::Sdl;
use sdl2::render::{ Canvas, Texture };

pub struct UI<'a> {
    pub sdl: Sdl,
    pub texture: Texture<'a>
}

impl<'a> UI<'a> {
    pub fn new() -> UI<'a> {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("rustboy", 640, 480)
            .position_centered()
            .opengl()
            .build()
            .unwrap();
        
        let mut canvas = window.into_canvas().build().unwrap();
        let texture_creator = canvas.texture_creator();
        let fmt = PixelFormatEnum::RGB24;
        let mut texture = texture_creator.create_texture_streaming(fmt, 256, 256).unwrap();

        // Create gradient
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..256 {
                for x in 0..256 {
                    let offset = y * pitch + x * 3;
                    buffer[offset] = x as u8;
                    buffer[offset + 1] = y as u8;
                    buffer[offset + 2] = 0;
                }
            }

            buffer[50 * pitch + 50 * 3] = 0;
            buffer[50 * pitch + 50 * 3 + 1] = 0;
            buffer[50 * pitch + 50 * 3 + 2] = 0;
        }).unwrap();

        canvas.clear();
        canvas.copy(&texture, None, Some(Rect::new(100, 100, 256, 256))).unwrap();
        canvas.present();

        let mut event_pump = sdl_context.event_pump().unwrap();

        UI { sdl: sdl_context, texture: texture }
    /*
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }*/
    }
}
