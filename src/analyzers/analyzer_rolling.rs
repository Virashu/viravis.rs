use std::collections::VecDeque;

use super::{
    utils::{fade_exponent, mean_abs, mean_nonzero, moving_average},
    Analyzer,
};

pub struct AnalyzerRolling {
    callback: Box<dyn Fn(Vec<f32>) + Send>,
    hist: VecDeque<f32>,
}

impl AnalyzerRolling {
    pub fn new(size: usize, callback: impl Fn(Vec<f32>) + Send + 'static) -> Self {
        Self {
            callback: Box::new(callback),
            hist: VecDeque::from(vec![0.0; size]),
        }
    }
}

impl Analyzer for AnalyzerRolling {
    fn analyze(&mut self, data: &[f32], _: &cpal::InputCallbackInfo) {
        let avg = mean_abs(data) * 1000.0;

        // Equalizing
        let mut baseline = mean_nonzero(self.hist.clone()).powi(2); // Can be 0.0 because of pow()

        if baseline == 0.0 {
            baseline = 1.0;
        }

        let avg_norm = avg / baseline * 100.0;

        // Rotate
        self.hist.push_front(avg_norm);
        self.hist.pop_back().unwrap();

        // Fade
        self.hist = moving_average(Vec::from(self.hist.clone()), 2);
        self.hist = fade_exponent(Vec::from(self.hist.clone()), 0.002);

        (self.callback)(Vec::from(self.hist.clone()));
    }
}
