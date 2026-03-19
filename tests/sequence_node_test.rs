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
