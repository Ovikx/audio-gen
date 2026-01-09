use std::{cell::RefCell, cmp::max, collections::HashSet, rc::Rc};

use audio_gen::{
    context::AudioContext,
    generator::SampleGenerator,
    node::float::FloatSource,
    sequencer::{GeneratorInterval, Sequencer},
};
use nalgebra::min;
use test_utils::threshold_eq_float32;

#[test]
fn test_non_overlapping_sample_existence() {
    const NUM_GENERATORS: u32 = 10;
    const FLOAT_VALUE: f32 = 1.0;
    let mut generator_intervals: Vec<GeneratorInterval> = vec![];
    let mut last_sample_index = 0;
    let mut expected_some_indices: HashSet<u32> = HashSet::new();

    for i in 0..NUM_GENERATORS {
        let generator = SampleGenerator::new(
            vec![Rc::new(RefCell::new(FloatSource::new(0, FLOAT_VALUE)))],
            AudioContext::new(1.),
        )
        .unwrap();

        let start_index = i * 4;
        let end_index = start_index + 2;
        for j in start_index..end_index + 1 {
            expected_some_indices.insert(j);
        }
        last_sample_index = max(last_sample_index, end_index);
        generator_intervals.push(GeneratorInterval::new(
            RefCell::new(generator),
            start_index,
            end_index,
        ));
    }

    let mut sequencer = Sequencer::new(generator_intervals);
    dbg!(last_sample_index);
    for i in 0..last_sample_index + 1 {
        let sample = sequencer.poll().unwrap(); // Stream shouldn't end in this loop
        if expected_some_indices.contains(&i) {
            debug_assert_eq!(sample, FLOAT_VALUE);
        } else {
            debug_assert_eq!(sample, 0.);
        }
    }

    let sample = sequencer.poll();
    assert!(sample.is_none()); // All generators should have been consumed in the earlier loop
}

#[test]
fn test_overlapping_sample_aggregation() {
    const NUM_GENERATORS: u32 = 7;
    const FLOAT_VALUE: f32 = 1.0;
    const INTERVAL_LENGTH: u32 = 5;
    let mut generator_intervals: Vec<GeneratorInterval> = vec![];

    for i in 0..NUM_GENERATORS {
        let generator = SampleGenerator::new(
            vec![Rc::new(RefCell::new(FloatSource::new(0, FLOAT_VALUE)))],
            AudioContext::new(1.),
        )
        .unwrap();
        generator_intervals.push(GeneratorInterval::new(
            RefCell::new(generator),
            i,
            i + INTERVAL_LENGTH - 1,
        ));
    }

    let mut sequencer = Sequencer::new(generator_intervals);
    for i in 0..NUM_GENERATORS + INTERVAL_LENGTH - 1 {
        let sample = sequencer.poll().unwrap(); // Stream shouldn't end in this loop
        if i < NUM_GENERATORS - 1 {
            let expected = min(i + 1, INTERVAL_LENGTH) as f32;
            assert!(
                threshold_eq_float32(sample, expected),
                "expected {}, got {}",
                expected,
                sample
            );
        } else {
            let expected = ((NUM_GENERATORS + INTERVAL_LENGTH - 1) - i) as f32;
            assert!(
                threshold_eq_float32(sample, expected),
                "expected {}, got {}",
                expected,
                sample
            );
        }
    }

    let sample = sequencer.poll();
    assert!(sample.is_none()); // All generators should have been consumed in the earlier loop
}
