use crate::{
    context::AudioContext,
    source::{NodeOutput, Source},
};

pub struct AbsoluteValue {
    id: usize,
    source_id: usize,
    dependency_ids: Vec<usize>,
}

impl AbsoluteValue {
    pub fn new(id: usize, source_id: usize) -> Self {
        AbsoluteValue {
            id,
            source_id,
            dependency_ids: vec![source_id],
        }
    }
}

impl Source for AbsoluteValue {
    fn poll(&mut self, _audio_context: &AudioContext, id_to_output: &NodeOutput) -> Option<f32> {
        id_to_output[self.source_id].map(|input| input.abs())
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
