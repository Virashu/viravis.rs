pub trait Analyzer {
    fn analyze(&mut self, _: &[f32], _: &::cpal::InputCallbackInfo) {}
}
