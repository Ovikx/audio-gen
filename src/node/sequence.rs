use std::{
    cmp,
    collections::{HashMap, HashSet},
};

use crate::{
    context::AudioContext,
    source::{NodeOutput, Source},
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct SourceInterval {
    pub source_id: usize,

    /// Inclusive bound
    pub start_index: u32,

    /// Inclusive bound
    pub end_index: u32,
}

impl SourceInterval {
    pub fn new(source_id: usize, start_index: u32, end_index: u32) -> Self {
        SourceInterval {
            source_id,
            start_index,
            end_index,
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
    active_intervals: HashSet<SourceInterval>,
    start_sorted_intervals: Vec<SourceInterval>,
    end_sorted_intervals: Vec<SourceInterval>,
    next_queue_index: usize,
    next_dequeue_index: usize,
    current_sample_index: u32,

    // Output buffer state
    source_id_to_output_buffer: Vec<Vec<f32>>,
    interval_to_output_buffer_read_idx: HashMap<SourceInterval, usize>,
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
        for interval in &source_intervals {
            dependency_ids.push(interval.source_id);
            let current_max = source_id_to_max_interval_length
                .get_mut(interval.source_id)
                .unwrap();
            *current_max = cmp::max(
                *current_max,
                (interval.end_index - interval.start_index + 1) as usize,
            );
        }

        // We know how many samples to buffer for each source, so we can preallocate all the required space
        let mut source_id_to_output_buffer = vec![vec![]; max_sources];
        for (source_id, max_interval_length) in source_id_to_max_interval_length.iter().enumerate()
        {
            let output_buffer = source_id_to_output_buffer.get_mut(source_id).unwrap();
            *output_buffer = Vec::with_capacity(*max_interval_length);
        }

        let mut start_sorted_intervals = source_intervals.clone();
        start_sorted_intervals.sort_by_key(|interval| interval.start_index);

        let mut end_sorted_intervals = source_intervals.clone();
        end_sorted_intervals.sort_by_key(|interval| interval.end_index);

        let interval_to_output_buffer_read_idx = source_intervals
            .clone()
            .into_iter()
            .map(|interval| (interval, 0))
            .collect();

        let unfilled_buffer_ids: HashSet<usize> = dependency_ids.iter().cloned().collect();

        SequenceNode {
            id,
            dependency_ids,
            start_sorted_intervals,
            end_sorted_intervals,
            active_intervals: HashSet::new(),
            next_queue_index: 0,
            next_dequeue_index: 0,
            current_sample_index: 0,
            source_id_to_output_buffer,
            interval_to_output_buffer_read_idx,
            unfilled_buffer_ids,
        }
    }

    fn poll(&mut self, id_to_output: &NodeOutput, sample_idx: usize) -> Option<f32> {
        // Dependency nodes don't wait until it's their turn to produce samples,
        // so we need to capture their output and use it later.
        self.unfilled_buffer_ids.retain(|source_id| {
            let output_buffer = self.source_id_to_output_buffer.get_mut(*source_id).unwrap();
            output_buffer.push(id_to_output[*source_id][sample_idx].unwrap_or(0.));
            output_buffer.len() < output_buffer.capacity()
        });

        while self.next_dequeue_index < self.end_sorted_intervals.len()
            && self.end_sorted_intervals[self.next_dequeue_index].end_index
                < self.current_sample_index
        {
            let dequeued_interval = self.end_sorted_intervals[self.next_dequeue_index];
            self.active_intervals.remove(&dequeued_interval);
            self.next_dequeue_index += 1;
        }

        while self.next_queue_index < self.start_sorted_intervals.len()
            && self.start_sorted_intervals[self.next_queue_index].start_index
                <= self.current_sample_index
        {
            self.active_intervals
                .insert(self.start_sorted_intervals[self.next_queue_index]);
            self.next_queue_index += 1;
        }

        // If there's nothing to queue or dequeue, we should just return nothing.
        if self.next_queue_index >= self.start_sorted_intervals.len()
            && self.next_dequeue_index >= self.end_sorted_intervals.len()
        {
            return None;
        }

        self.current_sample_index += 1;

        let sample_sum = self
            .active_intervals
            .iter()
            .map(|interval| {
                // TODO: Consider using a sparse vector instead of a map.
                // If audio graphs are assembled via `Graph`, then such a vector
                // will actually be dense.
                let output_buffer = self
                    .source_id_to_output_buffer
                    .get(interval.source_id)
                    .unwrap();
                let read_index = self
                    .interval_to_output_buffer_read_idx
                    .get_mut(interval)
                    .unwrap();
                let sample = output_buffer[*read_index];
                *read_index += 1;
                sample
            })
            .sum(); // The sum may go beyond the [-1.0, 1.0] range, so a clipping strategy would be required downstream.
        Some(sample_sum)
    }
}

impl Source for SequenceNode {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        _audio_context: &AudioContext,
        id_to_output: &NodeOutput,
        output: &mut [Option<f32>],
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
