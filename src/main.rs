use realfft::RealFftPlanner;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use std::error::Error;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, StreamConfig, SupportedBufferSize};

const CHUNK: usize = 960;

fn main() -> Result<(), Box<dyn Error>> {
    let host = cpal::default_host();

    let device = host.default_output_device().unwrap();

    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");

    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    let config = StreamConfig {
        channels: supported_config.channels(),
        sample_rate: supported_config.sample_rate(),
        buffer_size: BufferSize::Fixed(CHUNK as u32),
    };

    let mut real_planner = RealFftPlanner::<f32>::new();

    let callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let len = data.len();

        let r2c = real_planner.plan_fft_forward(len);

        // let mut indata = r2c.();
        let mut spectrum = r2c.make_output_vec();

        let mut indata = Vec::from(data);

        r2c.process(&mut indata, &mut spectrum).unwrap();
        println!("Got data. len: {}", spectrum.len());
    };

    let stream = device
        .build_input_stream(&config, callback, move |_err| {}, None)
        .unwrap();

    stream.play().unwrap();

    loop {}
}
