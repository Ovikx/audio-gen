use audio_gen::{context::audio_context::AudioContext, generator::SampleGenerator, graph};

use test_utils::threshold_eq_float32;

#[test]
fn test_saw_oscillator_node_sequence() {
    let mut graph = graph::Graph::new();
    let float_node_id = graph.insert_float_node(1.);
    graph.insert_saw_oscillator_node(float_node_id);

    let sample_rate = 4.;
    let mut generator =
        SampleGenerator::new(graph.nodes(), AudioContext::new(sample_rate)).unwrap();

    let num_sets = 100;
    let samples = generator.batch_poll(4 * num_sets + 1);
    let expected_samples: Vec<f32> = vec![0.5, 0.0, -0.5, 0.0]; // Index 1 is a discontinuity, so that value is not used for validation
    assert!(threshold_eq_float32(samples[0], 0.));
    dbg!(&samples);
    for i in 1..num_sets * 4 {
        if (i - 1) % 4 == 1 {
            assert!(
                threshold_eq_float32(samples[i as usize], 1.0)
                    || threshold_eq_float32(samples[i as usize], -1.0)
            );
            continue;
        }
        assert!(
            threshold_eq_float32(
                samples[i as usize],
                expected_samples[(((i - 1) as u32) % 4) as usize]
            ),
            "expected {}, got {} at index {}",
            expected_samples[(((i - 1) as u32) % 4) as usize],
            samples[i as usize],
            i
        );
    }
}
