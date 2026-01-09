use std::{cell::RefCell, collections::HashSet};

use crate::generator::SampleGenerator;

pub struct Sequencer {
    generator_intervals: Vec<GeneratorInterval>,
    active_generator_indices: HashSet<usize>,
    start_sorted_intervals: Vec<IndexedInterval>,
    end_sorted_intervals: Vec<IndexedInterval>,
    next_queue_index: usize,
    next_dequeue_index: usize,
    current_sample_index: u32,
}

pub struct GeneratorInterval {
    pub generator: RefCell<SampleGenerator>,

    /// Inclusive bound
    pub start_index: u32,

    /// Inclusive bound
    pub end_index: u32,
}

impl GeneratorInterval {
    pub fn new(generator: RefCell<SampleGenerator>, start_index: u32, end_index: u32) -> Self {
        GeneratorInterval {
            generator,
            start_index,
            end_index,
        }
    }
}

#[derive(Clone, Copy)]
struct IndexedInterval {
    pub generator_index: usize,
    pub start_index: u32,
    pub end_index: u32,
}
impl Sequencer {
    pub fn new(generator_intervals: Vec<GeneratorInterval>) -> Self {
        let indexed_intervals: Vec<IndexedInterval> = generator_intervals
            .iter()
            .enumerate()
            .map(|(index, interval)| IndexedInterval {
                generator_index: index,
                start_index: interval.start_index,
                end_index: interval.end_index,
            })
            .collect();

        let mut start_sorted_intervals = indexed_intervals.clone();
        start_sorted_intervals.sort_by_key(|interval| interval.start_index);

        let mut end_sorted_intervals = indexed_intervals.clone();
        end_sorted_intervals.sort_by_key(|interval| interval.end_index);

        Sequencer {
            generator_intervals,
            start_sorted_intervals,
            end_sorted_intervals,
            active_generator_indices: HashSet::new(),
            next_queue_index: 0,
            next_dequeue_index: 0,
            current_sample_index: 0,
        }
    }

    pub fn poll(&mut self) -> Option<f32> {
        while self.next_queue_index < self.start_sorted_intervals.len()
            && self.start_sorted_intervals[self.next_queue_index].start_index
                <= self.current_sample_index
        {
            self.active_generator_indices
                .insert(self.start_sorted_intervals[self.next_queue_index].generator_index);
            self.next_queue_index += 1;
        }

        while self.next_dequeue_index < self.end_sorted_intervals.len()
            && self.end_sorted_intervals[self.next_dequeue_index].end_index
                < self.current_sample_index
        {
            self.active_generator_indices
                .remove(&self.end_sorted_intervals[self.next_dequeue_index].generator_index);
            self.next_dequeue_index += 1;
        }

        // If there's nothing to queue or dequeue, we should just return nothing
        if self.next_queue_index >= self.start_sorted_intervals.len()
            && self.next_dequeue_index >= self.end_sorted_intervals.len()
        {
            return None;
        }

        self.current_sample_index += 1;

        let sample_sum = self
            .active_generator_indices
            .iter()
            .map(|index| {
                self.generator_intervals[*index]
                    .generator
                    .borrow_mut()
                    .poll()
            })
            .sum(); // The sum may go beyond the [-1.0, 1.0] range, so a clipping strategy would be required downstream
        Some(sample_sum)
    }
}
