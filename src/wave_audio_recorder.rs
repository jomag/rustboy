use std::{fs::File, io::BufWriter};

use crate::apu::apu::AudioRecorder;

pub struct WaveAudioRecorder {
    pub mono_writer: Option<hound::WavWriter<BufWriter<File>>>,
    pub gen1_writer: Option<hound::WavWriter<BufWriter<File>>>,
    pub gen2_writer: Option<hound::WavWriter<BufWriter<File>>>,
}

impl AudioRecorder for WaveAudioRecorder {
    fn mono(&mut self, sample: f32) {
        if let Some(ref mut wr) = self.mono_writer {
            wr.write_sample(sample);
        }
    }

    fn gen1(&mut self, sample: f32) {
        if let Some(ref mut wr) = self.gen1_writer {
            wr.write_sample(sample);
        }
    }

    fn gen2(&mut self, sample: f32) {
        if let Some(ref mut wr) = self.gen2_writer {
            wr.write_sample(sample);
        }
    }

    fn flush(&mut self) {
        if let Some(ref mut wr) = self.mono_writer {
            wr.flush();
        }

        if let Some(ref mut wr) = self.gen1_writer {
            wr.flush();
        }

        if let Some(ref mut wr) = self.gen2_writer {
            wr.flush();
        }
    }
}
