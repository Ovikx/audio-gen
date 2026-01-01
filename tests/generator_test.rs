use std::{cell::RefCell, rc::Rc};

use audio_gen::{
    context::AudioContext,
    generator::SampleGenerator,
    node::{float::FloatSource, sine_oscillator::SineOscillatorNode, sum::SumNode},
    scheduler::NodeExecutionSchedule,
};

use test_utils::threshold_eq_float32;

#[test]
fn test_basic_graph() {
    let float_source_node = Rc::new(RefCell::new(FloatSource::new(0, 1.)));
    let sine_oscillator_node = Rc::new(RefCell::new(SineOscillatorNode::new(1, 0)));
    let sum_node = Rc::new(RefCell::new(SumNode::new(2, 1, 1)));

    let sample_rate = 4.;
    let nodes: NodeExecutionSchedule = vec![float_source_node, sine_oscillator_node, sum_node];
    let mut generator = SampleGenerator::new(nodes, AudioContext::new(sample_rate)).unwrap();

    let num_sets = 100;
    let samples = generator.batch_poll(4 * num_sets + 1);
    let expected_samples: Vec<f32> = vec![2.0, 0.0, -2.0, 0.0];
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
