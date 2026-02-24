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
}

impl Source for NoiseNode {
    fn poll(&mut self, _audio_context: &AudioContext, _id_to_output: &NodeOutput) -> Option<f32> {
        Some(self.rng_gen.random_range(-1.0..=1.0))
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
