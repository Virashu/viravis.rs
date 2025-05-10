use std::error::Error;
use std::sync::{Arc, Mutex};

use clap::ValueEnum;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, InputCallbackInfo, SampleRate, StreamConfig};
use tracing::info;

use std::sync::mpsc::{channel, Receiver};

use crate::analyzers::{self, Analyzer};

const CHUNK: u32 = 960;
const DEFAULT_SAMPLE_RATE: u32 = 48_000;

#[derive(Clone, ValueEnum)]
pub enum AnalyzerMode {
    /// Fast Fourier Transform (columns by frequency)
    Fft,
    /// Wave-like effect by volume
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
    device: Arc<Mutex<cpal::Device>>,
    stream: Mutex<cpal::Stream>,
    analyzer: Arc<Mutex<dyn Analyzer>>,
    sample_rate: u32,
    callbacks: Vec<Box<dyn Fn(Vec<f32>)>>,
    channel: Receiver<Vec<f32>>,
}

impl Viravis {
    pub fn new(
        size: usize,
        mode: AnalyzerMode,
        sample_rate: Option<u32>,
    ) -> Result<Self, Box<dyn Error>> {
        let device = Self::get_device();
        info!("Selected device: `{}`", device.name()?);

        let sample_rate = sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE);
        let config = Self::get_config(&device, sample_rate);
        info!("Selected config: {:?}", config);

        let (tx, rx) = channel();

        let analyzer_cb = move |d| tx.send(d).unwrap();
        let analyzer: Arc<Mutex<dyn Analyzer>> = match mode {
            AnalyzerMode::Rolling => Arc::new(Mutex::new(analyzers::AnalyzerRolling::new(
                size,
                analyzer_cb,
            ))),
            AnalyzerMode::Fft => {
                Arc::new(Mutex::new(analyzers::AnalyzerFFT::new(size, analyzer_cb)))
            }
        };

        let stream = Self::build_stream(&device, &config, analyzer.clone());

        let vis = Self {
            device: Arc::new(Mutex::new(device)),
            stream: Mutex::new(stream),
            analyzer,
            sample_rate,
            callbacks: Vec::new(),
            channel: rx,
        };

        Ok(vis)
    }

    fn get_device() -> cpal::Device {
        cpal::default_host()
            .default_output_device()
            .expect("No default output device")
    }

    fn get_config(device: &cpal::Device, sample_rate: u32) -> StreamConfig {
        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");

        let supported_config = supported_configs_range
            .next()
            .expect("no supported config")
            .with_max_sample_rate();

        // TODO: Add check for support of passed sample rate

        StreamConfig {
            channels: supported_config.channels(),
            sample_rate: SampleRate(sample_rate),
            buffer_size: BufferSize::Fixed(CHUNK),
        }
    }

    fn build_stream(
        device: &cpal::Device,
        config: &StreamConfig,
        anal: Arc<Mutex<dyn Analyzer>>,
    ) -> cpal::Stream {
        let stream_cb = move |a: &[f32], b: &InputCallbackInfo| anal.lock().unwrap().analyze(a, b);
        device
            .build_input_stream(config, stream_cb, move |_err| {}, None)
            .expect("Failed to build Input stream")
    }

    fn device_check(&self) {
        let device = Self::get_device();
        let device_name = device.name().unwrap();

        if device_name == self.device.lock().unwrap().name().unwrap() {
            return;
        }

        tracing::info!("New device connected: `{}`", device_name);

        {
            let mut s_device = self.device.lock().unwrap();
            *s_device = device;
            let config = &Self::get_config(&s_device, self.sample_rate);
            let stream = Self::build_stream(&s_device, config, self.analyzer.clone());
            let mut s_stream = self.stream.lock().unwrap();
            *s_stream = stream;
            s_stream.play().unwrap();
        }
    }

    /// Start main loop
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        {
            self.stream.lock().unwrap().play()?;
        }

        loop {
            self.device_check();

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
