use crate::{
    context::audio_context::AudioContext,
    source::{LogicalTimestamp, SharedCachedFloatSource, Source},
};

pub struct SawOscillatorNode {
    frequency_source: SharedCachedFloatSource,
    current_time: f32,
}

impl SawOscillatorNode {
    pub fn new(frequency_source: SharedCachedFloatSource) -> Self {
        SawOscillatorNode {
            frequency_source,
            current_time: 0.,
        }
    }
}

impl Source<f32> for SawOscillatorNode {
    fn poll(&mut self, audio_context: &AudioContext, timestamp: LogicalTimestamp) -> Option<f32> {
        self.frequency_source
            .borrow_mut()
            .poll(audio_context, timestamp)
            .map(|f| {
                let sample = 2.0
                    * (f * self.current_time / audio_context.sample_rate
                        - (0.5 + f * self.current_time / audio_context.sample_rate).floor());
                self.current_time += 1.0;
                if self.current_time >= audio_context.sample_rate / f {
                    self.current_time -= (audio_context.sample_rate / f).floor();
                }
                sample
            })
    }
}
