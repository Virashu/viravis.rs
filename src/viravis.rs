use std::error::Error;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, InputCallbackInfo, StreamConfig};

use std::sync::mpsc::{channel, Receiver};

use crate::analyzers::{self, Analyzer};

const CHUNK: u32 = 960;

#[derive(Clone)]
pub enum AnalyzerMode {
    Fft,
    Rolling,
}

impl std::str::FromStr for AnalyzerMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rolling" => Ok(Self::Rolling),
            "fft" => Ok(Self::Fft),
            _ => Err(format!("Unknown log level: {s}")),
        }
    }
}

impl std::fmt::Display for AnalyzerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Fft => "fft",
            Self::Rolling => "rolling",
        };
        s.fmt(f)
    }
}

pub struct Viravis {
    stream: cpal::Stream,
    callbacks: Vec<Box<dyn Fn(Vec<f32>)>>,
    channel: Receiver<Vec<f32>>,
}

impl Viravis {
    pub fn new(size: usize, mode: AnalyzerMode) -> Result<Self, Box<dyn Error>> {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");

        let supported_config = supported_configs_range
            .next()
            .expect("no supported config")
            .with_max_sample_rate();

        let config = StreamConfig {
            channels: supported_config.channels(),
            sample_rate: supported_config.sample_rate(),
            buffer_size: BufferSize::Fixed(CHUNK),
        };

        let (tx, rx) = channel();

        let cb = move |d| tx.send(d).unwrap();

        let mut anal: Box<dyn Analyzer + Send> = match mode {
            AnalyzerMode::Rolling => Box::new(analyzers::AnalyzerRolling::new(size, cb)),
            AnalyzerMode::Fft => Box::new(analyzers::AnalyzerFFT::new(size, cb)),
        };

        let callback = move |a: &[f32], b: &InputCallbackInfo| anal.analyze(a, b);

        let stream = device.build_input_stream(&config, callback, move |_err| {}, None)?;

        let v = Self {
            stream,
            callbacks: Vec::new(),
            channel: rx,
        };

        Ok(v)
    }

    pub fn set_stream(&mut self, stream: cpal::Stream) {
        self.stream = stream;
    }

    /// Start main loop
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        self.stream.play()?;

        loop {
            let data = self.channel.recv().unwrap();
            self.update(data);
        }
    }

    pub fn add_callback<F>(&mut self, f: F)
    where
        F: Fn(Vec<f32>) + 'static,
    {
        self.callbacks.push(Box::new(f));
    }

    fn update(&self, data: Vec<f32>) {
        self.callbacks.iter().for_each(|c| (c)(data.clone()));
    }
}
