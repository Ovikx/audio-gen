use crate::{
    context::audio_context::AudioContext,
    source::{LogicalTimestamp, SharedCachedFloatSource, Source},
};

pub struct MultiplyNode {
    multiplicand_source: SharedCachedFloatSource,
    multiplier_source: SharedCachedFloatSource,
}

impl MultiplyNode {
    pub fn new(
        multiplicand_source: SharedCachedFloatSource,
        multiplier_source: SharedCachedFloatSource,
    ) -> Self {
        MultiplyNode {
            multiplicand_source,
            multiplier_source,
        }
    }
}

impl Source<f32> for MultiplyNode {
    fn poll(&mut self, audio_context: &AudioContext, timestamp: LogicalTimestamp) -> Option<f32> {
        self.multiplicand_source
            .borrow_mut()
            .poll(audio_context, timestamp)
            .map(|f| {
                f * self
                    .multiplier_source
                    .borrow_mut()
                    .poll(audio_context, timestamp)
                    .unwrap_or(1.0)
            })
    }
}
