use std::{
    cmp,
    collections::{HashMap, HashSet},
};

use crate::{
    context::AudioContext,
    source::{NodeOutput, Source},
};

#[derive(Clone, Copy, Debug)]
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

#[derive(Debug)]
struct OutputBuffer {
    buffer: Vec<f32>,
    read_index: usize,
}

/// Wrapper for a Sequencer.
/// Nodes cannot use samples directly from a Sequencer since it doesn't
/// implement `Source`. This node makes it possible to do so.
pub struct SequenceNode {
    id: usize,
    dependency_ids: Vec<usize>,

    // Sequencing state
    active_source_ids: HashSet<usize>,
    start_sorted_intervals: Vec<SourceInterval>,
    end_sorted_intervals: Vec<SourceInterval>,
    next_queue_index: usize,
    next_dequeue_index: usize,
    current_sample_index: u32,

    // Output buffer state
    source_id_to_output_buffer: HashMap<usize, OutputBuffer>,
    unfilled_buffer_ids: HashSet<usize>,
}

impl SequenceNode {
    pub fn new(id: usize, source_intervals: Vec<SourceInterval>) -> Self {
        let mut dependency_ids = vec![];

        // We only need to store enough samples for the longest interval for each source
        let mut source_id_to_max_interval_length = HashMap::new();
        for interval in &source_intervals {
            dependency_ids.push(interval.source_id);
            let current_max = source_id_to_max_interval_length
                .entry(interval.source_id)
                .or_insert(0);
            *current_max = cmp::max(
                *current_max,
                (interval.end_index - interval.start_index + 1) as usize,
            );
        }

        // We know how many samples to buffer for each source, so we can preallocate all the required space
        let mut source_id_to_output_buffer = HashMap::new();
        let mut source_id_to_output_buffer_index = HashMap::new();
        for (source_id, max_interval_length) in &source_id_to_max_interval_length {
            source_id_to_output_buffer.insert(
                *source_id,
                OutputBuffer {
                    buffer: Vec::with_capacity(*max_interval_length),
                    read_index: 0,
                },
            );
            source_id_to_output_buffer_index.insert(*source_id, 0);
        }

        let mut start_sorted_intervals = source_intervals.clone();
        start_sorted_intervals.sort_by_key(|interval| interval.start_index);

        let mut end_sorted_intervals = source_intervals.clone();
        end_sorted_intervals.sort_by_key(|interval| interval.end_index);

        let unfilled_buffer_ids: HashSet<usize> = dependency_ids.iter().cloned().collect();

        SequenceNode {
            id,
            dependency_ids,
            start_sorted_intervals,
            end_sorted_intervals,
            active_source_ids: HashSet::new(),
            next_queue_index: 0,
            next_dequeue_index: 0,
            current_sample_index: 0,
            source_id_to_output_buffer,
            unfilled_buffer_ids,
        }
    }

    fn poll(&mut self, id_to_output: &NodeOutput, sample_idx: usize) -> Option<f32> {
        // Dependency nodes don't wait until it's their turn to produce samples,
        // so we need to capture their output and use it later.
        self.unfilled_buffer_ids.retain(|source_id| {
            let output_buffer = self.source_id_to_output_buffer.get_mut(source_id).unwrap();
            let buffer_vec: &mut Vec<f32> = output_buffer.buffer.as_mut();
            buffer_vec.push(id_to_output[*source_id][sample_idx].unwrap_or(0.));
            buffer_vec.len() < buffer_vec.capacity()
        });

        while self.next_dequeue_index < self.end_sorted_intervals.len()
            && self.end_sorted_intervals[self.next_dequeue_index].end_index
                < self.current_sample_index
        {
            let dequeued_interval = self.end_sorted_intervals[self.next_dequeue_index];
            let output_buffer = self
                .source_id_to_output_buffer
                .get_mut(&dequeued_interval.source_id)
                .unwrap();
            output_buffer.read_index = 0; // The next interval using the same source will start from the beginning of its output buffer.

            self.active_source_ids.remove(&dequeued_interval.source_id);
            self.next_dequeue_index += 1;
        }

        while self.next_queue_index < self.start_sorted_intervals.len()
            && self.start_sorted_intervals[self.next_queue_index].start_index
                <= self.current_sample_index
        {
            self.active_source_ids
                .insert(self.start_sorted_intervals[self.next_queue_index].source_id);
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
            .active_source_ids
            .iter()
            .map(|source_id| {
                let output_buffer = self.source_id_to_output_buffer.get_mut(source_id).unwrap();
                let sample = output_buffer.buffer[output_buffer.read_index];
                output_buffer.read_index += 1;
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
