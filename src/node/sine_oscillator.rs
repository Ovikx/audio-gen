use crate::{
    context::audio_context::AudioContext,
    source::{LogicalTimestamp, SharedCachedFloatSource, Source},
};
use std::f32::consts::PI;

pub struct SineOscillatorNode {
    frequency_source: SharedCachedFloatSource,
    phase: f32,
}

impl SineOscillatorNode {
    pub fn new(frequency_source: SharedCachedFloatSource) -> Self {
        SineOscillatorNode {
            frequency_source,
            phase: 0.,
        }
    }
}

impl Source<f32> for SineOscillatorNode {
    fn poll(&mut self, audio_context: &AudioContext, timestamp: LogicalTimestamp) -> Option<f32> {
        self.frequency_source
            .borrow_mut()
            .poll(audio_context, timestamp)
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
