use crate::{
    context::AudioContext,
    source::{NodeOutput, Source},
};

use rand::{Rng, SeedableRng, rngs::StdRng};

pub struct NoiseNode {
    id: usize,
    rng_gen: StdRng,
    dependency_ids: Vec<usize>,
}

impl NoiseNode {
    pub fn new(id: usize, seed: u64) -> Self {
        NoiseNode {
            id,
            rng_gen: StdRng::seed_from_u64(seed),
            dependency_ids: vec![],
        }
    }

    fn poll(&mut self) -> Option<f32> {
        Some(self.rng_gen.random_range(-1.0..=1.0))
    }
}

impl Source for NoiseNode {
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
