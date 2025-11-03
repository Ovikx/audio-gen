use crate::{
    context::audio_context::AudioContext,
    node::source::{SharedFloatSource, Source},
};
use std::f32::consts::PI;

pub struct SineOscillatorNode {
    frequency_source: SharedFloatSource,
    phase: f32,
}

impl SineOscillatorNode {
    pub fn new(frequency_source: SharedFloatSource) -> Self {
        SineOscillatorNode {
            frequency_source,
            phase: 0.,
        }
    }
}

impl Source<f32> for SineOscillatorNode {
    fn poll(&mut self, audio_context: &AudioContext) -> Option<f32> {
        self.frequency_source
            .borrow_mut()
            .poll(audio_context)
            .map(|f| {
                let sample = self.phase.sin();
                self.phase += f * 2.0 * PI / audio_context.sample_rate;
                if self.phase > 2.0 * PI {
                    self.phase -= 2.0 * PI;
                }
                sample
            })
    }
}
