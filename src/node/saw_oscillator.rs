use crate::{
    context::AudioContext,
    source::{NodeOutput, Source},
};

pub struct SawOscillatorNode {
    id: usize,
    frequency_source_id: usize,
    dependency_ids: Vec<usize>,
    phase: f32,
}

impl SawOscillatorNode {
    pub fn new(id: usize, frequency_source_id: usize) -> Self {
        SawOscillatorNode {
            id,
            frequency_source_id,
            dependency_ids: vec![frequency_source_id],
            phase: 0.,
        }
    }
}

impl Source for SawOscillatorNode {
    fn poll(&mut self, audio_context: &AudioContext, id_to_output: &NodeOutput) -> Option<f32> {
        id_to_output[self.frequency_source_id].map(|f| {
            let sample = 2.0 * self.phase - 1.0;
            self.phase += f / audio_context.sample_rate;
            self.phase = self.phase.fract();
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
