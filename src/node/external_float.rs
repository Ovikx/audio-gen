use crate::{
    context::AudioContext,
    input_buffer::SharedExternalInputBuffer,
    source::{NodeOutput, Source},
};

pub struct ExternalFloatNode {
    id: usize,
    input_buffer: SharedExternalInputBuffer,
    input_buffer_index: usize,
    dependency_ids: Vec<usize>,
}

impl ExternalFloatNode {
    pub fn new(
        id: usize,
        input_buffer: SharedExternalInputBuffer,
        input_buffer_index: usize,
    ) -> Self {
        ExternalFloatNode {
            id,
            input_buffer,
            input_buffer_index,
            dependency_ids: vec![],
        }
    }
}

impl Source for ExternalFloatNode {
    fn poll(&mut self, _audio_context: &AudioContext, _id_to_output: &NodeOutput) -> Option<f32> {
        Some(self.input_buffer.borrow().f32[self.input_buffer_index])
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
