use std::sync::{Arc, RwLock};

use cpal::{
    traits::{DeviceTrait as _, HostTrait as _},
    Stream,
};

const GAIN_MINUS_INFINITY: f32 = 0.00001;

pub struct AudioBuffer {
    samples: [f32; 4096],
    index: usize,
}

impl Default for AudioBuffer {
    fn default() -> Self {
        Self {
            samples: [0.0; 4096],
            index: 0,
        }
    }
}

impl AudioBuffer {
    pub fn push(&mut self, value: f32) {
        self.samples[self.index] = value;
        self.index += 1;
        if self.index >= self.samples.len() {
            self.index = 0;
        }
    }

    pub fn rms(&self) -> f32 {
        let s: f32 = self.samples.iter().map(|v| v * v).sum();

        (s / self.samples.len() as f32).sqrt()
    }
}

pub fn create_stream() -> (Stream, Arc<RwLock<AudioBuffer>>) {
    let buf = Arc::new(RwLock::new(AudioBuffer::default()));
    let buf_s = buf.clone();

    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();
    for c in host.input_devices().unwrap() {
        println!("{}", c.name().unwrap());
    }
    let config = device.default_input_config().unwrap();
    let stream = device
        .build_input_stream(
            &config.config(),
            move |d: &[f32], _i| {
                let mut b = buf_s.write().unwrap();
                for v in d {
                    b.push(*v);
                }
            },
            |e| {
                eprintln!("Error occured in the input stream: {e}");
            },
            None,
        )
        .unwrap();

    (stream, buf)
}

pub fn gain_to_db(gain: f32) -> f32 {
    return gain.abs().max(GAIN_MINUS_INFINITY).log10() * 20.0;
}
