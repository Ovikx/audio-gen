use audio_gen::{context::AudioContext, generator::SampleGenerator, graph};

use test_utils::threshold_eq_float32;

#[test]
fn test_saw_oscillator_node_sequence() {
    let mut graph = graph::Graph::new(false);
    let float_node_id = graph.float_node(1.);
    graph.saw_oscillator_node(float_node_id);

    let num_sets = 100;
    let sample_rate = 4.;
    let mut generator = SampleGenerator::new(
        graph.nodes(),
        AudioContext::new(sample_rate),
        4 * num_sets + 1,
    )
    .unwrap();

    let samples = generator.batch_poll();
    let expected_samples: Vec<f32> = vec![-1.0, -0.5, 0.0, 0.5]; // Index 1 is a discontinuity, so that value is not used for validation
    for i in 0..num_sets * 4 {
        assert!(threshold_eq_float32(
            samples[i as usize],
            expected_samples[((i as u32) % 4) as usize]
        ));
    }
}
