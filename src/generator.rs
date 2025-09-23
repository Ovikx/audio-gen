use crate::{context::audio_context::AudioContext, node::source::Source};

pub struct SampleGenerator {
    sample_source: Box<dyn Source<f32>>,
    audio_context: AudioContext,
}

impl SampleGenerator {
    pub fn new(
        sample_source: Box<dyn Source<f32>>,
        audio_context: AudioContext,
    ) -> SampleGenerator {
        SampleGenerator {
            sample_source,
            audio_context,
        }
    }

    pub fn generate_samples(&mut self, num_samples: u32) -> Vec<f32> {
        let mut samples = vec![];
        for _ in 0..num_samples {
            samples.push(self.sample_source.poll(&self.audio_context).unwrap_or(0.0));
        }
        samples
    }
}
