use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, SampleFormat, Stream, StreamConfig,
};
use ringbuf::{Producer, RingBuffer};

pub trait AudioRecorder {
    fn mono(&mut self, sample: f32);
    fn gen1(&mut self, sample: f32);
    fn gen2(&mut self, sample: f32);
    fn flush(&mut self);
}

pub struct AudioPlayer {
    stream: Option<Stream>,
    pub producer: Option<Producer<i16>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        AudioPlayer {
            stream: None,
            producer: None,
        }
    }

    pub fn setup(&mut self) {
        let buf = RingBuffer::<i16>::new(((48000 * 10) / 60) as usize);
        let (producer, mut consumer) = buf.split();
        self.producer = Some(producer);

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

        let err_fn = |err| eprintln!("an error occured on the output audio stream: {}", err);
        let sample_format = config.sample_format();
        let config: StreamConfig = config.into();

        let channels = config.channels as usize;

        let mut next_value = move || match consumer.pop() {
            Some(sample) => (sample as f32) / 32768.0,
            None => 0.0,
        };

        fn write_beep<T: Sample>(
            output: &mut [T],
            channels: usize,
            next_sample: &mut dyn FnMut() -> f32,
        ) {
            for frame in output.chunks_mut(channels) {
                let value: T = cpal::Sample::from::<f32>(&next_sample());
                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        }

        let stream = match sample_format {
            SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    write_beep::<f32>(data, channels, &mut next_value)
                },
                err_fn,
            ),

            SampleFormat::I16 => device.build_output_stream(
                &config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    write_beep::<i16>(data, channels, &mut next_value)
                },
                err_fn,
            ),

            SampleFormat::U16 => device.build_output_stream(
                &config,
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    write_beep::<u16>(data, channels, &mut next_value)
                },
                err_fn,
            ),
        }
        .unwrap();

        stream.play().unwrap();
        self.stream = Some(stream);
    }
}
