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

    fn poll(&mut self, input: f32) -> Option<f32> {
        Some(input)
    }
}

impl Source for ExternalFloatNode {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        _audio_context: &AudioContext,
        _id_to_output: &NodeOutput,
        output: &mut [Option<f32>],
    ) {
        let input = self.input_buffer.lock().unwrap().f32[self.input_buffer_index];

        for idx in 0..num_samples {
            output[idx] = self.poll(input);
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
