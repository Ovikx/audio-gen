use crate::{context::audio_context::AudioContext, node::source::Source};

pub struct MultiplyNode<S: Source<f32>, G: Source<f32>> {
    sample_source: S,
    factor_source: G,
}

impl<S: Source<f32>, G: Source<f32>> MultiplyNode<S, G> {
    pub fn new(sample_source: S, factor_source: G) -> Self {
        MultiplyNode {
            sample_source,
            factor_source,
        }
    }
}

impl<S: Source<f32>, G: Source<f32>> Source<f32> for MultiplyNode<S, G> {
    fn poll(&mut self, audio_context: &AudioContext) -> Option<f32> {
        self.sample_source
            .poll(audio_context)
            .map(|f| f * self.factor_source.poll(audio_context).unwrap_or(1.0))
    }
}
