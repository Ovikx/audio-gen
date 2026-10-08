use std::collections::VecDeque;

use crate::source::Source;

pub struct DelayNode {
    id: usize,
    source_id: usize,
    dependency_ids: Vec<usize>,
    num_skipped_samples: usize,
    sample_history: VecDeque<f32>,
}

impl DelayNode {
    pub fn new(id: usize, source_id: usize, num_skipped_samples: usize) -> Self {
        DelayNode {
            id,
            source_id,
            dependency_ids: vec![source_id],
            num_skipped_samples,
            sample_history: VecDeque::with_capacity(num_skipped_samples),
        }
    }

    pub fn poll(&mut self, sample: f32) -> f32 {
        if self.num_skipped_samples == 0 {
            return sample;
        }

        // 0.s should be delayed as well. For instance, 0.s in a sequence node should be respected as real samples.
        let next_sample = if self.sample_history.len() == self.num_skipped_samples {
            self.sample_history
                .pop_front()
                .expect("expected sample history queue to be non-empty")
        } else {
            0.
        };

        self.sample_history.push_back(sample);
        next_sample
    }
}

impl Source for DelayNode {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        _audio_context: &crate::context::AudioContext,
        id_to_output: &crate::source::NodeOutput,
        output: &mut [f32],
    ) {
        for idx in 0..num_samples {
            output[idx] = self.poll(id_to_output[self.source_id][idx])
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_delay() {
        let (expected_sequence, actual_sequence) = sequence_pair_from_delay(0);
        assert_eq!(expected_sequence, actual_sequence);
    }

    #[test]
    fn test_single_sample_delay() {
        let (expected_sequence, actual_sequence) = sequence_pair_from_delay(1);
        assert_eq!(expected_sequence, actual_sequence);
    }

    #[test]
    fn test_multi_sample_delay() {
        let (expected_sequence, actual_sequence) = sequence_pair_from_delay(4);
        assert_eq!(expected_sequence, actual_sequence);
    }

    fn sequence_pair_from_delay(delay: usize) -> (Vec<f32>, Vec<f32>) {
        let mut node = DelayNode::new(0, 0, delay);
        let mut input_sequence: Vec<f32> = vec![1., 0., 0., 2., 3., 4.];

        let mut expected_sequence = input_sequence.clone();
        expected_sequence.splice(..0, vec![0.; delay].into_iter());

        input_sequence.extend(vec![0.; delay]);
        let mut actual_sequence = vec![];
        for sample in input_sequence {
            actual_sequence.push(node.poll(sample));
        }

        (expected_sequence, actual_sequence)
    }
}
