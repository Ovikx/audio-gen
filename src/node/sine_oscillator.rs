use crate::{
    context::audio_context::AudioContext,
    source::{NodeOutput, Source},
};
use std::f32::consts::PI;

pub struct SineOscillatorNode {
    id: usize,
    frequency_source_id: usize,
    dependency_ids: Vec<usize>,
    phase: f32,
}

impl SineOscillatorNode {
    pub fn new(id: usize, frequency_source_id: usize) -> Self {
        SineOscillatorNode {
            id,
            frequency_source_id,
            dependency_ids: vec![frequency_source_id],
            phase: 0.,
        }
    }
}

impl Source for SineOscillatorNode {
    fn poll(&mut self, audio_context: &AudioContext, id_to_output: &NodeOutput) -> Option<f32> {
        id_to_output[self.frequency_source_id].map(|f| {
            let sample = self.phase.sin();
            self.phase += f * 2.0 * PI / audio_context.sample_rate;
            if self.phase > 2.0 * PI {
                self.phase -= 2.0 * PI;
            }
            sample
        })
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
