use crate::{
    context::AudioContext,
    source::{NodeOutput, Source},
};

pub struct MultiplyNode {
    id: usize,
    multiplicand_source_id: usize,
    multiplier_source_id: usize,
    dependency_ids: Vec<usize>,
}

impl MultiplyNode {
    pub fn new(id: usize, multiplicand_source_id: usize, multiplier_source_id: usize) -> Self {
        MultiplyNode {
            id,
            multiplicand_source_id,
            multiplier_source_id,
            dependency_ids: vec![multiplicand_source_id, multiplier_source_id],
        }
    }

    fn poll(&mut self, multiplicand: Option<f32>, multiplier: Option<f32>) -> Option<f32> {
        Some(multiplicand.unwrap_or(1.) * multiplier.unwrap_or(1.))
    }
}

impl Source for MultiplyNode {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        _audio_context: &AudioContext,
        id_to_output: &NodeOutput,
        output: &mut [Option<f32>],
    ) {
        for idx in 0..num_samples {
            output[idx] = self.poll(
                id_to_output[self.multiplicand_source_id][idx],
                id_to_output[self.multiplier_source_id][idx],
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
