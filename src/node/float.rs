use crate::{
    context::AudioContext,
    source::{NodeOutput, Source},
};

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

    fn poll(&mut self) -> Option<f32> {
        Some(self.value)
    }
}

impl Source for FloatSource {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        _audio_context: &AudioContext,
        _id_to_output: &NodeOutput,
        output: &mut [Option<f32>],
    ) {
        for idx in 0..num_samples {
            output[idx] = self.poll();
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
