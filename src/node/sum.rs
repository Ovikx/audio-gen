use crate::{
    context::AudioContext,
    source::{NodeOutput, Source},
};

pub struct SumNode {
    id: usize,
    augend_source_id: usize,
    addend_source_id: usize,
    dependency_ids: Vec<usize>,
}

impl SumNode {
    pub fn new(id: usize, augend_source_id: usize, addend_source_id: usize) -> Self {
        SumNode {
            id,
            augend_source_id,
            addend_source_id,
            dependency_ids: vec![augend_source_id, addend_source_id],
        }
    }

    fn poll(&mut self, augend: Option<f32>, addend: Option<f32>) -> Option<f32> {
        Some(augend.unwrap_or(0.) + addend.unwrap_or(0.))
    }
}

impl Source for SumNode {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        _audio_context: &AudioContext,
        id_to_output: &NodeOutput,
        output: &mut [Option<f32>],
    ) {
        for idx in 0..num_samples {
            output[idx] = self.poll(
                id_to_output[self.augend_source_id][idx],
                id_to_output[self.addend_source_id][idx],
            );
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
