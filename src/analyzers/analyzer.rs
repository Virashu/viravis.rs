pub trait Analyzer: Send {
    fn analyze(&mut self, _: &[f32], _: &cpal::InputCallbackInfo) {}
}
