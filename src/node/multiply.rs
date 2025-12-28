use crate::{
    context::audio_context::AudioContext,
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
}

impl Source for MultiplyNode {
    fn poll(&mut self, _audio_context: &AudioContext, id_to_output: &NodeOutput) -> Option<f32> {
        Some(
            id_to_output[self.multiplicand_source_id].unwrap_or(1.)
                * id_to_output[self.multiplier_source_id].unwrap_or(1.),
        )
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
