use crate::{
    context::audio_context::AudioContext,
    source::{NodeOutput, Source},
};

pub struct SawOscillatorNode {
    id: usize,
    frequency_source_id: usize,
    dependency_ids: Vec<usize>,
    current_time: f32,
}

impl SawOscillatorNode {
    pub fn new(id: usize, frequency_source_id: usize) -> Self {
        SawOscillatorNode {
            id,
            frequency_source_id,
            dependency_ids: vec![frequency_source_id],
            current_time: 0.,
        }
    }
}

impl Source for SawOscillatorNode {
    fn poll(&mut self, audio_context: &AudioContext, id_to_output: &NodeOutput) -> Option<f32> {
        id_to_output[self.frequency_source_id].map(|f| {
            let sample = 2.0
                * (f * self.current_time / audio_context.sample_rate
                    - (0.5 + f * self.current_time / audio_context.sample_rate).floor());
            self.current_time += 1.0;
            if self.current_time >= audio_context.sample_rate / f {
                self.current_time -= (audio_context.sample_rate / f).floor();
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
