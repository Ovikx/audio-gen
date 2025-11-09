use crate::{
    context::audio_context::AudioContext,
    source::{LogicalTimestamp, Source},
};

pub struct Float32Source {
    value: f32,
}

impl Float32Source {
    pub fn new(value: f32) -> Self {
        Float32Source { value }
    }
}

impl Source<f32> for Float32Source {
    fn poll(&mut self, _audio_context: &AudioContext, _timestamp: LogicalTimestamp) -> Option<f32> {
        Some(self.value)
    }
}
