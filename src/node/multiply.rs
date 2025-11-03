use crate::{
    context::audio_context::AudioContext,
    node::source::{SharedFloatSource, Source},
};

pub struct MultiplyNode {
    multiplicand_source: SharedFloatSource,
    multiplier_source: SharedFloatSource,
}

impl MultiplyNode {
    pub fn new(
        multiplicand_source: SharedFloatSource,
        multiplier_source: SharedFloatSource,
    ) -> Self {
        MultiplyNode {
            multiplicand_source,
            multiplier_source,
        }
    }
}

impl Source<f32> for MultiplyNode {
    fn poll(&mut self, audio_context: &AudioContext) -> Option<f32> {
        self.multiplicand_source
            .borrow_mut()
            .poll(audio_context)
            .map(|f| {
                f * self
                    .multiplier_source
                    .borrow_mut()
                    .poll(audio_context)
                    .unwrap_or(1.0)
            })
    }
}
