use std::f32::consts::PI;

use crate::{
    context::AudioContext,
    source::{NodeOutput, Source},
};

pub struct SVFNode {
    id: usize,
    filter_type: FilterType,
    sample_source_id: usize,
    frequency_cutoff_source_id: usize,
    resonance_source_id: usize,
    dependency_ids: Vec<usize>,

    // Accumulator states
    hp: f32,
    bp: f32,
    lp: f32,
}

pub enum FilterType {
    HighPass,
    BandPass,
    LowPass,
}

impl SVFNode {
    pub fn new(
        id: usize,
        filter_type: FilterType,
        sample_source_id: usize,
        frequency_cutoff_source_id: usize,
        resonance_source_id: usize,
    ) -> Self {
        SVFNode {
            id,
            filter_type,
            sample_source_id,
            frequency_cutoff_source_id,
            resonance_source_id,
            dependency_ids: vec![
                sample_source_id,
                frequency_cutoff_source_id,
                resonance_source_id,
            ],
            hp: 0.,
            bp: 0.,
            lp: 0.,
        }
    }

    fn poll(
        &mut self,
        audio_context: &AudioContext,
        frequency_cutoff: Option<f32>,
        resonance: Option<f32>,
        sample: Option<f32>,
    ) -> Option<f32> {
        frequency_cutoff
            .zip(resonance)
            .zip(sample)
            .map(|((frequency_cutoff, resonance), sample)| {
                let frequency_control =
                    2.0 * (PI * frequency_cutoff / audio_context.sample_rate).sin();
                let damping = 1.0 / resonance;

                self.hp = sample - self.lp - (damping * self.bp);
                self.bp += frequency_control * self.hp;
                self.lp += frequency_control * self.bp;

                match self.filter_type {
                    FilterType::HighPass => self.hp,
                    FilterType::BandPass => self.bp,
                    FilterType::LowPass => self.lp,
                }
            })
    }
}

impl Source for SVFNode {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        audio_context: &AudioContext,
        id_to_output: &NodeOutput,
        output: &mut [Option<f32>],
    ) {
        for idx in 0..num_samples {
            output[idx] = self.poll(
                audio_context,
                id_to_output[self.frequency_cutoff_source_id][idx],
                id_to_output[self.resonance_source_id][idx],
                id_to_output[self.sample_source_id][idx],
            );
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
