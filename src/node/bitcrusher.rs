use crate::source::Source;

pub struct BitcrusherNode {
    id: usize,
    dependency_ids: Vec<usize>,

    sample_source_id: usize,
    downsampling_factor_source_id: usize,
    resolution_factor_source_id: usize,

    // Downsampling requires replaying the same sample for some amount of time
    phase: f32,
    sample_to_keep: Option<f32>,
}

impl BitcrusherNode {
    pub fn new(
        id: usize,
        sample_source_id: usize,
        downsampling_factor_source_id: usize,
        resolution_factor_source_id: usize,
    ) -> Self {
        BitcrusherNode {
            id,
            sample_source_id,
            downsampling_factor_source_id,
            resolution_factor_source_id,
            dependency_ids: vec![
                sample_source_id,
                downsampling_factor_source_id,
                resolution_factor_source_id,
            ],
            phase: 1.0,
            sample_to_keep: None,
        }
    }

    pub fn poll(
        &mut self,
        sample: Option<f32>,
        downsampling_factor: Option<f32>,
        resolution_factor: Option<f32>,
    ) -> Option<f32> {
        // Extremely low downsampling and resolution factors aren't
        // practical, so let's use a slightly high lower bound. This also
        // eliminates divide-by-zero errors.
        let downsampling_factor = downsampling_factor.unwrap_or(1.0).clamp(0.01, 1.0);
        let resolution_factor = resolution_factor.unwrap_or(1.0).clamp(0.01, 1.0);

        if self.phase >= 1.0 {
            self.phase -= 1.0;
            self.sample_to_keep = sample;
        }
        self.phase += downsampling_factor;

        let reduced_resolution_sample = if let Some(real_sample) = self.sample_to_keep {
            Some(reduce_sample_resolution(real_sample, resolution_factor))
        } else {
            None
        };

        reduced_resolution_sample
    }
}

impl Source for BitcrusherNode {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        _audio_context: &crate::context::AudioContext,
        id_to_output: &crate::source::NodeOutput,
        output: &mut [Option<f32>],
    ) {
        for idx in 0..num_samples {
            output[idx] = self.poll(
                id_to_output[self.sample_source_id][idx],
                id_to_output[self.downsampling_factor_source_id][idx],
                id_to_output[self.resolution_factor_source_id][idx],
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

fn reduce_sample_resolution(sample: f32, resolution_factor: f32) -> f32 {
    if resolution_factor == 1.0 {
        return sample;
    }

    let bucket_size = 1.0 - resolution_factor;
    let num_buckets = sample / bucket_size;
    let reduced_resolution_sample = if num_buckets >= 0.0 {
        num_buckets.ceil() * bucket_size
    } else {
        num_buckets.floor() * bucket_size
    };
    reduced_resolution_sample
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reduce_sample_resolution() {
        // No resolution reduction
        assert_eq!(reduce_sample_resolution(1.0, 1.0), 1.0);
        assert_eq!(reduce_sample_resolution(0.75, 1.0), 0.75);
        assert_eq!(reduce_sample_resolution(-1.0, 1.0), -1.0);
        assert_eq!(reduce_sample_resolution(-0.75, 1.0), -0.75);

        // Half resolution reduction
        assert_eq!(reduce_sample_resolution(0.25, 0.5), 0.5);
        assert_eq!(reduce_sample_resolution(-0.25, 0.5), -0.5);
        assert_eq!(reduce_sample_resolution(0.75, 0.5), 1.0);
        assert_eq!(reduce_sample_resolution(-0.75, 0.5), -1.0);
    }
}
