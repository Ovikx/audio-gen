use crate::{
    context::audio_context::AudioContext,
    node::source::{SharedFloatSource, Source},
};

pub struct SumNode {
    augend_source: SharedFloatSource,
    addend_source: SharedFloatSource,
}

impl SumNode {
    pub fn new(augend_source: SharedFloatSource, addend_source: SharedFloatSource) -> Self {
        SumNode {
            augend_source,
            addend_source,
        }
    }
}

impl Source<f32> for SumNode {
    fn poll(&mut self, audio_context: &AudioContext) -> Option<f32> {
        let augend = self.augend_source.borrow_mut().poll(audio_context);
        let addend = self.addend_source.borrow_mut().poll(audio_context);
        if let (Some(augend), Some(addend)) = (augend, addend) {
            return Some(augend + addend);
        }
        None
    }
}
