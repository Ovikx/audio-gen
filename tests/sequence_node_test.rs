use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use audio_gen::{
    context::AudioContext,
    generator::SampleGenerator,
    graph::Graph,
    node::FloatSource,
    sequencer::{GeneratorInterval, Sequencer},
};
use nalgebra::min;
use test_utils::threshold_eq_float32;

#[test]
fn test_overlapping_sample_aggregation() {
    const NUM_GENERATORS: u32 = 7;
    const FLOAT_VALUE: f32 = 1.0;
    const INTERVAL_LENGTH: u32 = 5;
    let mut generator_intervals: Vec<GeneratorInterval> = vec![];

    for i in 0..NUM_GENERATORS {
        let generator = SampleGenerator::new(
            vec![Arc::new(Mutex::new(FloatSource::new(0, FLOAT_VALUE)))],
            AudioContext::new(1.),
        )
        .unwrap();
        generator_intervals.push(GeneratorInterval::new(
            RefCell::new(generator),
            i,
            i + INTERVAL_LENGTH - 1,
        ));
    }

    let sequencer = Sequencer::new(generator_intervals);
    let mut master_graph = Graph::new();
    let sequence_node_id = master_graph.sequence_node(Arc::new(Mutex::new(sequencer)));
    let multiplier_id = master_graph.float_node(2.);
    master_graph.multiply_node(sequence_node_id, multiplier_id);

    let mut master_generator =
        SampleGenerator::new(master_graph.nodes(), AudioContext::new(1.)).unwrap();

    // let master_generator = SampleGenerator::new(vec![Arc::new(Mutex::new(SequenceNode::new(0, Arc::new(Mutex::new()))))])
    for i in 0..NUM_GENERATORS + INTERVAL_LENGTH - 1 {
        let sample = master_generator.poll(); // Stream shouldn't end in this loop
        if i < NUM_GENERATORS - 1 {
            let expected = 2. * min(i + 1, INTERVAL_LENGTH) as f32;
            assert!(
                threshold_eq_float32(sample, expected),
                "expected {}, got {}",
                expected,
                sample
            );
        } else {
            let expected = 2. * ((NUM_GENERATORS + INTERVAL_LENGTH - 1) - i) as f32;
            assert!(
                threshold_eq_float32(sample, expected),
                "expected {}, got {}",
                expected,
                sample
            );
        }
    }
}
