use std::{cmp, collections::HashSet};

use crate::{
    context::AudioContext,
    source::{NodeOutput, Source},
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct SourceInterval {
    pub source_id: usize,

    /// Inclusive bound
    pub start_idx: u32,

    /// Inclusive bound
    pub end_idx: u32,
}

impl SourceInterval {
    pub fn new(source_id: usize, start_idx: u32, end_idx: u32) -> Self {
        SourceInterval {
            source_id,
            start_idx,
            end_idx,
        }
    }
}

/// Wrapper for a Sequencer.
/// Nodes cannot use samples directly from a Sequencer since it doesn't
/// implement `Source`. This node makes it possible to do so.
pub struct SequenceNode {
    id: usize,
    dependency_ids: Vec<usize>,

    // Sequencing state
    source_intervals: Vec<SourceInterval>,
    active_interval_idxs: Vec<usize>,
    start_sorted_interval_idxs: Vec<usize>,
    end_sorted_interval_idxs: Vec<usize>,
    next_queue_idx: usize,
    next_dequeue_idx: usize,
    current_sample_idx: u32,

    // Output buffer state
    source_id_to_output_buffer: Vec<Vec<f32>>,
    interval_id_to_output_buffer_read_idx: Vec<usize>,
    unfilled_buffer_ids: HashSet<usize>,
}

impl SequenceNode {
    pub fn new(id: usize, source_intervals: Vec<SourceInterval>) -> Self {
        let mut dependency_ids = vec![];
        let max_sources = source_intervals
            .iter()
            .map(|interval| interval.source_id)
            .max()
            .unwrap()
            + 1;

        // We only need to store enough samples for the longest interval for each source
        let mut source_id_to_max_interval_length: Vec<usize> = vec![0; max_sources];
        for interval in source_intervals.iter() {
            dependency_ids.push(interval.source_id);
            let current_max = source_id_to_max_interval_length
                .get_mut(interval.source_id)
                .unwrap();
            *current_max = cmp::max(
                *current_max,
                (interval.end_idx - interval.start_idx + 1) as usize,
            );
        }

        // We know how many samples to buffer for each source, so we can preallocate all the required space
        let mut source_id_to_output_buffer = vec![vec![]; max_sources];
        for (source_id, max_interval_length) in source_id_to_max_interval_length.iter().enumerate()
        {
            let output_buffer = source_id_to_output_buffer.get_mut(source_id).unwrap();
            *output_buffer = Vec::with_capacity(*max_interval_length);
        }

        let mut enumerated_start_sorted_intervals: Vec<(usize, SourceInterval)> =
            source_intervals.clone().into_iter().enumerate().collect();
        enumerated_start_sorted_intervals.sort_by_key(|(_, interval)| interval.start_idx);
        let start_sorted_interval_idxs = enumerated_start_sorted_intervals
            .iter()
            .map(|(idx, _)| *idx)
            .collect();

        let mut enumerated_end_sorted_intervals: Vec<(usize, SourceInterval)> =
            source_intervals.clone().into_iter().enumerate().collect();
        enumerated_end_sorted_intervals.sort_by_key(|(_, interval)| interval.end_idx);
        let end_sorted_interval_idxs = enumerated_end_sorted_intervals
            .iter()
            .map(|(idx, _)| *idx)
            .collect();

        let interval_id_to_output_buffer_read_idx = vec![0; source_intervals.len()];

        let unfilled_buffer_ids: HashSet<usize> = dependency_ids.iter().cloned().collect();

        SequenceNode {
            id,
            dependency_ids,
            source_intervals,
            start_sorted_interval_idxs,
            end_sorted_interval_idxs,
            active_interval_idxs: vec![],
            next_queue_idx: 0,
            next_dequeue_idx: 0,
            current_sample_idx: 0,
            source_id_to_output_buffer,
            interval_id_to_output_buffer_read_idx,
            unfilled_buffer_ids,
        }
    }

    fn poll(&mut self, id_to_output: &NodeOutput, sample_idx: usize) -> f32 {
        // Dependency nodes don't wait until it's their turn to produce samples,
        // so we need to capture their output and use it later.
        self.unfilled_buffer_ids.retain(|source_id| {
            let output_buffer = self.source_id_to_output_buffer.get_mut(*source_id).unwrap();
            output_buffer.push(id_to_output[*source_id][sample_idx]);
            output_buffer.len() < output_buffer.capacity()
        });

        while self.next_dequeue_idx < self.end_sorted_interval_idxs.len()
            && self.source_intervals[self.end_sorted_interval_idxs[self.next_dequeue_idx]].end_idx
                < self.current_sample_idx
        {
            let dequeued_interval_idx = self.end_sorted_interval_idxs[self.next_dequeue_idx];
            self.active_interval_idxs
                .retain(|idx| *idx != dequeued_interval_idx);
            self.next_dequeue_idx += 1;
        }

        while self.next_queue_idx < self.start_sorted_interval_idxs.len()
            && self.source_intervals[self.start_sorted_interval_idxs[self.next_queue_idx]].start_idx
                <= self.current_sample_idx
        {
            self.active_interval_idxs
                .push(self.start_sorted_interval_idxs[self.next_queue_idx]);
            self.next_queue_idx += 1;
        }

        // If there's nothing to queue or dequeue, we should just return nothing.
        if self.next_queue_idx >= self.start_sorted_interval_idxs.len()
            && self.next_dequeue_idx >= self.end_sorted_interval_idxs.len()
        {
            return 0.;
        }

        self.current_sample_idx += 1;

        let sample_sum = self
            .active_interval_idxs
            .iter()
            .map(|interval_idx| {
                // TODO: Consider using a sparse vector instead of a map.
                // If audio graphs are assembled via `Graph`, then such a vector
                // will actually be dense.
                let output_buffer = self
                    .source_id_to_output_buffer
                    .get(self.source_intervals[*interval_idx].source_id)
                    .unwrap();
                let read_idx = self
                    .interval_id_to_output_buffer_read_idx
                    .get_mut(*interval_idx)
                    .unwrap();
                let sample = output_buffer[*read_idx];
                *read_idx += 1;
                sample
            })
            .sum(); // The sum may go beyond the [-1.0, 1.0] range, so a clipping strategy would be required downstream.
        sample_sum
    }
}

impl Source for SequenceNode {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        _audio_context: &AudioContext,
        id_to_output: &NodeOutput,
        output: &mut [f32],
    ) {
        for idx in 0..num_samples {
            output[idx] = self.poll(id_to_output, idx);
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
