use std::sync::{Arc, RwLock};

use cpal::{
    traits::{DeviceTrait as _, HostTrait as _, StreamTrait as _},
    BuildStreamError, DefaultStreamConfigError, Device, DeviceNameError, DevicesError, Host,
    PlayStreamError, Stream,
};
use thiserror::Error;

const GAIN_MINUS_INFINITY: f32 = 0.00001;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("There is no audio devices in the system")]
    NoDevice,

    #[error("Could not get default stream config: {0}")]
    DefaultStreamConfig(#[from] DefaultStreamConfigError),

    #[error("Could not build audio stream: {0}")]
    BuildStream(#[from] BuildStreamError),

    #[error("Could not list input devices: {0}")]
    Devices(#[from] DevicesError),

    #[error("Could not get device name: {0}")]
    DeviceName(#[from] DeviceNameError),

    #[error("Could not start audio stream: {0}")]
    PlayStream(#[from] PlayStreamError),
}

#[allow(unused)]
pub struct AudioInterface {
    buf: Arc<RwLock<AudioBuffer>>,
    stream: Stream,
    device: Device,
    host: Host,
}

impl AudioInterface {
    pub fn new(name: Option<&str>) -> Result<Self, AudioError> {
        let buf = Arc::<RwLock<AudioBuffer>>::default();
        let buf_s = buf.clone();

        let host = cpal::default_host();
        let device = match name {
            Some(name) => host
                .input_devices()?
                .filter_map(|d| d.name().ok().filter(|n| n == name).map(|_| d))
                .next(),
            None => host.default_input_device(),
        }
        .ok_or(AudioError::NoDevice)?;

        let config = device.default_input_config()?;
        let stream = device.build_input_stream(
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
        )?;

        stream.play()?;

        Ok(Self {
            buf,
            stream,
            device,
            host,
        })
    }

    pub fn available_inputs(&self) -> Result<Vec<String>, AudioError> {
        Ok(self
            .host
            .input_devices()?
            .filter(|d| d.default_input_config().is_ok())
            .filter_map(|d| d.name().ok())
            .collect::<Vec<String>>())
    }

    pub fn current_name(&self) -> Result<String, AudioError> {
        Ok(self.device.name()?)
    }

    pub fn rms(&self) -> f32 {
        self.buf.read().unwrap().rms()
    }
}

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

pub fn gain_to_db(gain: f32) -> f32 {
    gain.abs().max(GAIN_MINUS_INFINITY).log10() * 20.0
}
