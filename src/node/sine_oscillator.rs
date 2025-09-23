use crate::{context::audio_context::AudioContext, node::source::Source};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Serialize, Deserialize)]
pub struct SineOscillatorNode<F: Source<f32>> {
    frequency_source: F,
    phase: f32,
}

impl<F: Source<f32>> SineOscillatorNode<F> {
    pub fn new(frequency: F) -> Self {
        SineOscillatorNode {
            frequency_source: frequency,
            phase: 0.,
        }
    }
}

impl<F: Source<f32>> Source<f32> for SineOscillatorNode<F> {
    fn poll(&mut self, audio_context: &AudioContext) -> Option<f32> {
        self.frequency_source.poll(audio_context).map(|f| {
            let sample = self.phase.sin();
            self.phase += f * 2.0 * PI / audio_context.sample_rate;
            if self.phase > 2.0 * PI {
                self.phase -= 2.0 * PI;
            }
            sample
        })
    }
}
