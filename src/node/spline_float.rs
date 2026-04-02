use crate::{
    context::AudioContext,
    math::{interpolator::Interpolator, spline_polynomial::Point},
    source::{NodeOutput, Source},
};

pub struct SplineFloatNode {
    id: usize,
    frequency_source_id: usize,
    dependency_ids: Vec<usize>,
    interpolator: Interpolator,
}

impl SplineFloatNode {
    pub fn new(id: usize, frequency_source_id: usize, points: Vec<Point>) -> Self {
        let interpolator = Interpolator::new(points, true);
        SplineFloatNode {
            id,
            frequency_source_id,
            dependency_ids: vec![frequency_source_id],
            interpolator,
        }
    }

    fn poll(&mut self, audio_context: &AudioContext, frequency: Option<f32>) -> Option<f32> {
        frequency.map(|f| self.interpolator.next(1.0 / f, audio_context))
    }
}

impl Source for SplineFloatNode {
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
