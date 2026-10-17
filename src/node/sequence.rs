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
    next_queue_idx: usize,
    current_sample_idx: u32,

    // Output buffer state
    source_id_to_output_buffer: Vec<Vec<f32>>,
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

        let unfilled_buffer_ids: HashSet<usize> = dependency_ids.iter().cloned().collect();

        SequenceNode {
            id,
            dependency_ids,
            source_intervals,
            start_sorted_interval_idxs,
            active_interval_idxs: vec![],
            next_queue_idx: 0,
            current_sample_idx: 0,
            source_id_to_output_buffer,
            unfilled_buffer_ids,
        }
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
        // We rely on arithmetic addition instead of assignments, so we need to
        // clear in between batches.
        output[..num_samples].fill(0.0);

        // Dependency nodes will eagerly produce samples, so we have to capture
        // them before they are overwritten.
        self.unfilled_buffer_ids.retain(|source_id| {
            let output_buffer = self.source_id_to_output_buffer.get_mut(*source_id).unwrap();
            let num_to_copy = (output_buffer.capacity() - output_buffer.len()).min(num_samples);

            output_buffer.extend_from_slice(&id_to_output[*source_id][..num_to_copy]);
            output_buffer.len() < output_buffer.capacity()
        });

        self.active_interval_idxs.retain(|interval_idx| {
            self.source_intervals[*interval_idx].end_idx >= self.current_sample_idx
        });

        while self.next_queue_idx < self.start_sorted_interval_idxs.len()
            && self.source_intervals[self.start_sorted_interval_idxs[self.next_queue_idx]].start_idx
                <= (self.current_sample_idx as usize + num_samples) as u32
        {
            self.active_interval_idxs
                .push(self.start_sorted_interval_idxs[self.next_queue_idx]);
            self.next_queue_idx += 1;
        }

        for interval_idx in &self.active_interval_idxs {
            let interval = self.source_intervals[*interval_idx];
            let output_start_idx =
                (interval.start_idx as i32 - self.current_sample_idx as i32).max(0) as usize;
            let output_end_idx =
                (interval.end_idx - self.current_sample_idx + 1).min(num_samples as u32) as usize;

            let source_start_idx =
                (self.current_sample_idx as i32 - interval.start_idx as i32).max(0) as usize;
            let source_end_idx = source_start_idx + (output_end_idx - output_start_idx);

            let source_output_buffer = &self.source_id_to_output_buffer[interval.source_id];
            for (output_elem, &source_elem) in output[output_start_idx..output_end_idx]
                .iter_mut()
                .zip(source_output_buffer[source_start_idx..source_end_idx].iter())
            {
                *output_elem += source_elem;
            }
        }

        self.current_sample_idx += num_samples as u32;
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
