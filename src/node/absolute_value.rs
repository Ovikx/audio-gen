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

    fn poll(&mut self, input: Option<f32>) -> Option<f32> {
        input.map(|input| input.abs())
    }
}

impl Source for AbsoluteValue {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        _audio_context: &AudioContext,
        id_to_output: &NodeOutput,
        output: &mut [Option<f32>],
    ) {
        for idx in 0..num_samples {
            output[idx] = self.poll(id_to_output[self.source_id][idx]);
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
