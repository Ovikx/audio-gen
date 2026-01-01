use audio_gen::{context::AudioContext, generator::SampleGenerator, graph};

use test_utils::threshold_eq_float32;

#[test]
fn test_spline_node_sequence() {
    let mut graph = graph::Graph::new();
    let float_node_id = graph.insert_float_node(1.);
    graph.insert_spline_float_node(float_node_id, vec![(0.0, 0.0), (1.0, 1.0)]);

    let sample_rate = 4.;
    let mut generator =
        SampleGenerator::new(graph.nodes(), AudioContext::new(sample_rate)).unwrap();

    let num_sets = 100;
    let samples = generator.batch_poll(4 * num_sets + 1);
    let expected_samples: Vec<f32> = vec![0.25, 0.5, 0.75, 0.0];
    assert!(threshold_eq_float32(samples[0], 0.));
    dbg!(&samples);
    for i in 1..num_sets * 4 {
        dbg!(
            samples[i as usize],
            expected_samples[((i as u32) % 4) as usize]
        );
        assert!(threshold_eq_float32(
            samples[i as usize],
            expected_samples[(((i - 1) as u32) % 4) as usize]
        ));
    }
}
