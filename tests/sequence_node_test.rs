use audio_gen::{
    context::AudioContext, generator::SampleGenerator, graph::Graph, node::SourceInterval,
};
use nalgebra::min;
use test_utils::threshold_eq_float32;

#[test]
fn test_overlapping_sample_aggregation() {
    const NUM_GENERATORS: u32 = 7;
    const FLOAT_VALUE: f32 = 1.0;
    const INTERVAL_LENGTH: u32 = 5;
    const SAMPLE_RATE: f32 = 1.;
    let mut intervals: Vec<SourceInterval> = vec![];
    let mut graph = Graph::new(false);

    for i in 0..NUM_GENERATORS {
        let float_id = graph.float_node(FLOAT_VALUE);
        intervals.push(SourceInterval::new(float_id, i, i + INTERVAL_LENGTH - 1));
    }

    graph.sequence_node(intervals);
    let mut generator = SampleGenerator::new(
        graph.nodes(),
        AudioContext::new(SAMPLE_RATE),
        (NUM_GENERATORS + INTERVAL_LENGTH - 1) as usize,
    )
    .unwrap();

    let samples = generator.batch_poll();
    for i in 0..NUM_GENERATORS + INTERVAL_LENGTH - 1 {
        let sample = samples[i as usize]; // Stream shouldn't end in this loop
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
}

#[test]
fn test_consecutive_same_source() {
    const SAMPLE_RATE: f32 = 1.;
    const NUM_INTERVALS: u32 = 5;
    const INTERVAL_LENGTH: u32 = 2;
    let mut graph = Graph::new(true);
    let float_id = graph.float_node(1.0);

    let mut intervals: Vec<SourceInterval> = vec![];
    for i in 0..NUM_INTERVALS {
        intervals.push(SourceInterval::new(
            float_id,
            i * INTERVAL_LENGTH + 1,
            i * INTERVAL_LENGTH + INTERVAL_LENGTH,
        ));
    }

    graph.sequence_node(intervals);
    let mut generator = SampleGenerator::new(
        graph.nodes(),
        AudioContext::new(SAMPLE_RATE),
        (INTERVAL_LENGTH * NUM_INTERVALS) as usize,
    )
    .unwrap();

    let samples = generator.batch_poll();
    for sample in samples.into_iter().skip(1) {
        assert_eq!(sample, 1.0);
    }
}

#[test]
fn test_endpoint_overlap_same_source() {
    const SAMPLE_RATE: f32 = 1.;
    const NUM_INTERVALS: u32 = 5;
    const INTERVAL_LENGTH: u32 = 2;
    let mut graph = Graph::new(true);
    let float_id = graph.float_node(1.0);

    let mut intervals: Vec<SourceInterval> = vec![];
    for i in 0..NUM_INTERVALS {
        intervals.push(SourceInterval::new(
            float_id,
            i * INTERVAL_LENGTH,
            i * INTERVAL_LENGTH + INTERVAL_LENGTH,
        ));
    }

    graph.sequence_node(intervals);
    let mut generator = SampleGenerator::new(
        graph.nodes(),
        AudioContext::new(SAMPLE_RATE),
        (INTERVAL_LENGTH * NUM_INTERVALS) as usize,
    )
    .unwrap();

    let samples = generator.batch_poll();
    for (i, sample) in samples.into_iter().skip(1).enumerate() {
        assert_eq!(sample, ((i % 2) + 1) as f32);
    }
}
