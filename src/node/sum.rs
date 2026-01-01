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
}

impl Source for SumNode {
    fn poll(&mut self, _audio_context: &AudioContext, id_to_output: &NodeOutput) -> Option<f32> {
        Some(
            id_to_output[self.augend_source_id].unwrap_or(0.)
                + id_to_output[self.addend_source_id].unwrap_or(0.),
        )
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
