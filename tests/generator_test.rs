use std::{cell::RefCell, rc::Rc};

use audio_gen::{
    context::audio_context::AudioContext,
    generator::SampleGenerator,
    node::{float::Float32Source, sine_oscillator::SineOscillatorNode, sum::SumNode},
    source::CachedFloatSource,
};

use test_utils::threshold_eq_float32;
// use common::util::threshold_eq_float32;

#[test]
fn test_cached_sequence() {
    let cached_oscillator_node = Rc::new(RefCell::new(CachedFloatSource::new(Box::new(
        SineOscillatorNode::new(Rc::new(RefCell::new(CachedFloatSource::new(Box::new(
            Float32Source::new(1.),
        ))))),
    ))));

    let sum_node = Rc::new(RefCell::new(CachedFloatSource::new(Box::new(
        SumNode::new(
            Rc::clone(&cached_oscillator_node),
            Rc::clone(&cached_oscillator_node),
        ),
    ))));

    let sample_rate = 4.;
    let mut generator = SampleGenerator::new(Rc::clone(&sum_node), AudioContext::new(sample_rate));

    let num_sets = 100;
    let samples = generator.generate_samples(4 * num_sets + 1);
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
