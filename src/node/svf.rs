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
}

impl Source for SVFNode {
    fn poll(&mut self, audio_context: &AudioContext, id_to_output: &NodeOutput) -> Option<f32> {
        id_to_output[self.frequency_cutoff_source_id]
            .zip(id_to_output[self.resonance_source_id])
            .zip(id_to_output[self.sample_source_id])
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

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
