use crate::{
    context::AudioContext,
    math::{interpolator::Interpolator, spline_polynomial::Point},
    source::{NodeOutput, Source},
};

pub struct ADSRNode {
    id: usize,
    sample_source_id: usize,
    dependency_ids: Vec<usize>,
    interpolator: Interpolator,
    duration: f32,
}

impl ADSRNode {
    pub fn new(
        id: usize,
        sample_source_id: usize,
        duration: f32,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    ) -> Self {
        let mut points: Vec<Point> = vec![];

        // Avoid conflicting points at (0, 0)
        if attack > 0. {
            points.push((0., 0.));
        }
        points.push((attack / duration, 1.0));
        if attack + decay > attack {
            points.push(((attack + decay) / duration, sustain));
        }

        points.push(((duration - release) / duration, 1.));
        // Avoid conflicting points at (x=1)
        if release > 0. {
            points.push((1., 0.));
        }

        let interpolator = Interpolator::new(points, false);
        ADSRNode {
            id,
            sample_source_id,
            interpolator,
            dependency_ids: vec![sample_source_id],
            duration,
        }
    }

    fn poll(&mut self, audio_context: &AudioContext, sample: Option<f32>) -> Option<f32> {
        sample.map(|s| s * self.interpolator.next(self.duration, audio_context))
    }
}

impl Source for ADSRNode {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        audio_context: &AudioContext,
        id_to_output: &NodeOutput,
        output: &mut [Option<f32>],
    ) {
        for idx in 0..num_samples {
            output[idx] = self.poll(audio_context, id_to_output[self.sample_source_id][idx]);
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
