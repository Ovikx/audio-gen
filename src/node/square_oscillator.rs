use crate::{
    context::AudioContext,
    source::{NodeOutput, Source},
};

pub struct SquareOscillatorNode {
    id: usize,
    frequency_source_id: usize,
    dependency_ids: Vec<usize>,
    current_time: f32,
}

impl SquareOscillatorNode {
    pub fn new(id: usize, frequency_source_id: usize) -> Self {
        SquareOscillatorNode {
            id,
            frequency_source_id,
            dependency_ids: vec![frequency_source_id],
            current_time: 0.,
        }
    }
}

impl Source for SquareOscillatorNode {
    fn poll(&mut self, audio_context: &AudioContext, id_to_output: &NodeOutput) -> Option<f32> {
        id_to_output[self.frequency_source_id].map(|f| {
            let sample: f32;
            if self.current_time < 0.5 {
                sample = 1.;
            } else {
                sample = -1.;
            }
            self.current_time += f / audio_context.sample_rate;
            self.current_time -= 1.0 * self.current_time.floor();
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
