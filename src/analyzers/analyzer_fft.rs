use realfft::RealFftPlanner;

use super::{
    utils::{moving_average, smooth_directional},
    Analyzer,
};

const MOVING_AVERAGE_WINDOW: usize = 3;
const SMOOTHING_SPEED_UP: f32 = 0.7;
const SMOOTHING_SPEED_DOWN: f32 = 0.1;

pub struct AnalyzerFFT {
    callback: Box<dyn Fn(Vec<f32>) + Send>,
    fft_planner: RealFftPlanner<f32>,
    hist: Vec<f32>, //  Previous FFT
}

impl AnalyzerFFT {
    pub fn new(size: usize, callback: impl Fn(Vec<f32>) + Send + 'static) -> Self {
        Self {
            callback: Box::new(callback),
            fft_planner: RealFftPlanner::<f32>::new(),
            hist: vec![0.0; size],
        }
    }
}

impl Analyzer for AnalyzerFFT {
    fn analyze(&mut self, data: &[f32], _: &::cpal::InputCallbackInfo) {
        let len = data.len();
        let mut in_data = Vec::from(data);

        let r2c = self.fft_planner.plan_fft_forward(len);
        let mut spectrum = r2c.make_output_vec();

        r2c.process(&mut in_data, &mut spectrum).unwrap();

        let fft: Vec<f32> = spectrum.iter().map(|n| n.re.abs() * 20.0).collect();

        // Smoothen
        let s = moving_average(fft, MOVING_AVERAGE_WINDOW); // horizontal
        let s = smooth_directional(
            self.hist.clone(),
            s,
            SMOOTHING_SPEED_UP,
            SMOOTHING_SPEED_DOWN,
        );

        self.hist = s.clone();

        (self.callback)(s);
    }
}
