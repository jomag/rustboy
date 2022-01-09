// Previously, the emulator was synced on audio. While this made the
// emulator run at a very predictable and correct speed, it could cause
// screen tearing and it's not super simple to combine with something
// like ImGui that runs best at a fixed fps.
//
// My current plan is to instead dynamically adjust the sample rate
// depending on the current FPS.
//
// Here's some discussion about how that can be done:
// https://forums.nesdev.org/viewtopic.php?f=3&t=15405
//
// Here's a sample app based on that discussion:
// https://github.com/jslepicka/audio_sync/blob/master/audio_sync.cpp

use std::sync::{Arc, Condvar, Mutex};

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::Sdl;

use crate::ui::FPS;

pub const SAMPLE_RATE: u32 = 48_000;

struct AudioBuffer {
    buf: Arc<Mutex<[i16; 48_000]>>,
    pair: Arc<(Mutex<bool>, Condvar)>,
}

impl AudioCallback for AudioBuffer {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        let mut i: usize = 0;
        let data = self.buf.lock().unwrap();
        for x in out.iter_mut() {
            *x = data[i];
            i = i + 1;
        }

        let &(ref _lock, ref cvar) = &*self.pair;
        cvar.notify_one();
    }
}

fn setup_audio(sdl_context: Sdl) -> Result<(), String> {
    let audio_subsystem = sdl_context.audio()?;
    let audio_buffer: Arc<Mutex<[i16; SAMPLE_RATE as usize]>> =
        Arc::new(Mutex::new([0; SAMPLE_RATE as usize]));
    let audio_sync_pair = Arc::new((Mutex::new(false), Condvar::new()));

    let samples_per_frame: u32 = (SAMPLE_RATE * 100) / (FPS * 100.0) as u32;

    let desired_audio_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE as i32),
        channels: Some(1),
        samples: Some(samples_per_frame as u16),
    };

    // FIXME: validate that the received sample rate matches the desired rate
    let audio_device = audio_subsystem
        .open_playback(None, &desired_audio_spec, |_spec| AudioBuffer {
            buf: audio_buffer.clone(),
            pair: audio_sync_pair.clone(),
        })
        .unwrap();

    // Start playback
    audio_device.resume();

    return Ok(());
}
