use crate::{context::AudioContext, source::Source};

pub struct FloatSource {
    id: usize,
    value: f32,
    dependency_ids: Vec<usize>,
}

impl FloatSource {
    pub fn new(id: usize, value: f32) -> Self {
        FloatSource {
            id,
            value,
            dependency_ids: vec![],
        }
    }
}

impl Source for FloatSource {
    fn poll(
        &mut self,
        _audio_context: &AudioContext,
        _id_to_output: &crate::source::NodeOutput,
    ) -> Option<f32> {
        Some(self.value)
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
