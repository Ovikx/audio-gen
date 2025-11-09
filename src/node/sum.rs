use crate::{
    context::audio_context::AudioContext,
    source::Source,
    source::{LogicalTimestamp, SharedCachedFloatSource},
};

pub struct SumNode {
    augend_source: SharedCachedFloatSource,
    addend_source: SharedCachedFloatSource,
}

impl SumNode {
    pub fn new(
        augend_source: SharedCachedFloatSource,
        addend_source: SharedCachedFloatSource,
    ) -> Self {
        SumNode {
            augend_source,
            addend_source,
        }
    }
}

impl Source<f32> for SumNode {
    fn poll(&mut self, audio_context: &AudioContext, timestamp: LogicalTimestamp) -> Option<f32> {
        let augend = self
            .augend_source
            .borrow_mut()
            .poll(audio_context, timestamp);
        let addend = self
            .addend_source
            .borrow_mut()
            .poll(audio_context, timestamp);
        if let (Some(augend), Some(addend)) = (augend, addend) {
            return Some(augend + addend);
        }
        None
    }
}
