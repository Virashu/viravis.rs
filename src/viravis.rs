use std::error::Error;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, InputCallbackInfo, SampleRate, StreamConfig};

use std::sync::mpsc::{channel, Receiver};

use crate::analyzers::{self, Analyzer};

const CHUNK: u32 = 960;
const SAMPLE_RATE: u32 = 48_000;

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
            _ => Err(format!("Unknown mode: {s}")),
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
    pub fn new(
        size: usize,
        mode: AnalyzerMode,
        sample_rate_opt: Option<u32>,
    ) -> Result<Self, Box<dyn Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No default output device");

        log::info!("Selected device: `{}`", device.name()?);

        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");

        let supported_config = supported_configs_range
            .next()
            .expect("no supported config")
            .with_max_sample_rate();

        let sample_rate: u32;

        // TODO: Add check for support of passed sample rate

        if let Some(s_r) = sample_rate_opt {
            sample_rate = s_r;
        } else {
            sample_rate = SAMPLE_RATE;
        }

        let config = StreamConfig {
            channels: supported_config.channels(),
            sample_rate: SampleRate(sample_rate),
            buffer_size: BufferSize::Fixed(CHUNK),
        };

        log::info!("Selected config: {:?}", config);

        let (tx, rx) = channel();

        let cb = move |d| tx.send(d).unwrap();

        let mut anal: Box<dyn Analyzer> = match mode {
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
            let data = self.channel.recv()?;
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
