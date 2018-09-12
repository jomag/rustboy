
extern crate sdl2;

use sdl2::audio::{AudioSpecDesired, AudioQueue};
use std::time::Duration;
use std::thread::sleep;
use std::option::Option;

pub struct SquareWaveSoundGenerator {
    duty: u32,
    period: u32,
    ctr: u32,
    volume: i16
}

impl SquareWaveSoundGenerator {
    pub fn new() -> Self {
        SquareWaveSoundGenerator {
            duty: 2,
            period: 0,
            ctr: 0,
            volume: 16384
        }
    }

    pub fn generate(&mut self, samples: usize) -> Vec<i16> {
        let mut volume = self.volume;
        let mut result = Vec::new();

        for _ in 0..samples {
            if self.ctr % 10 == 0 {
                if (volume > 5) {
                    volume = volume - 20;
                }
            }

            if self.period == 0 {
                result.push(0);
            } else {
                self.ctr += 1;
                if self.ctr >= self.period {
                    self.ctr = 0;
                }

                if self.ctr * 8 < self.period * self.duty {
                    result.push(volume);
                } else {
                    result.push(-volume);
                }
            }
        }

        self.volume = volume;
        result
    }
}

pub struct AudioProcessingUnit {
    pub s1: SquareWaveSoundGenerator,
    pub queue: AudioQueue<i16>
}

impl AudioProcessingUnit {
    pub fn new(queue: AudioQueue<i16>) -> Self {
        AudioProcessingUnit {
            s1: SquareWaveSoundGenerator::new(),
            queue: queue,
        }
    }

    pub fn run(&mut self, timeout: u32) {
        let period = 16;
        let samples = ((44100 * period) / 800) as usize;
        for _ in 0..(timeout / period) {
            let snd = self.s1.generate(samples);
            self.queue.queue(&snd);
            sleep(Duration::from_millis(period as u64));
        }
    }
}

pub fn sound_test() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1),
        samples: None
    };

    let queue = audio_subsystem.open_queue::<i16, _>(None, &desired_spec).unwrap();
    queue.resume();
    let mut apu = AudioProcessingUnit::new(queue);

    let c4: f32 = 261.61;
    let d4: f32 = 293.66;
    let e4: f32 = 329.63;
    let f4: f32 = 349.23;
    let g4: f32 = 392.00;
    let a4: f32 = 440.00;
    let b4: f32 = 493.88;

    let song = [
        (e4, 150, 50),
        (c4, 150, 50),
        (d4, 150, 50),

        (g4, 100, 0),
        (e4, 200, 0),
        (e4, 100, 0),
        (c4, 150, 50),
        (d4, 150, 50),
        (g4, 150, 50)
    ];

    loop {
        for note in song.iter() {
            let freq = note.0;
            let play = note.1;
            let pause = note.2;

            println!("Play {}", freq);
            apu.s1.duty = apu.s1.duty + 1;
            if apu.s1.duty == 6 { apu.s1.duty = 2 }
            apu.s1.period = ((1.0 / freq) * 44100.0) as u32;
            apu.s1.volume = 16384;
            println!(" period: {}", apu.s1.period);
            apu.run(play);

            println!("...");
            apu.s1.period = 0;
            apu.run(pause);
        }
    }
}
