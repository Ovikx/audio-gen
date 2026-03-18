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

    fn poll(&mut self, audio_context: &AudioContext, frequency: Option<f32>) -> Option<f32> {
        frequency.map(|f| {
            let sample = 2.0 * self.phase - 1.0;
            self.phase += f / audio_context.sample_rate;
            self.phase = self.phase.fract();
            sample
        })
    }
}

impl Source for SawOscillatorNode {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        audio_context: &AudioContext,
        id_to_output: &NodeOutput,
        output: &mut [Option<f32>],
    ) {
        for idx in 0..num_samples {
            output[idx] = self.poll(audio_context, id_to_output[self.frequency_source_id][idx]);
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
