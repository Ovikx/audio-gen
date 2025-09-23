use crate::{context::audio_context::AudioContext, node::source::Source};

pub struct SumNode<S: Source<f32>, T: Source<f32>> {
    sample_source1: S,
    sample_source2: T,
}

impl<S: Source<f32>, T: Source<f32>> SumNode<S, T> {
    pub fn new(sample_source1: S, sample_source2: T) -> Self {
        SumNode {
            sample_source1,
            sample_source2,
        }
    }
}

impl<S: Source<f32>, T: Source<f32>> Source<f32> for SumNode<S, T> {
    fn poll(&mut self, audio_context: &AudioContext) -> Option<f32> {
        let sample1 = self.sample_source1.poll(audio_context);
        let sample2 = self.sample_source2.poll(audio_context);
        if let (Some(sample1), Some(sample2)) = (sample1, sample2) {
            Some(sample1 + sample2)
        } else {
            None
        }
    }
}
