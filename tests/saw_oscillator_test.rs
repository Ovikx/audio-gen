use std::{cell::RefCell, rc::Rc};

use audio_gen::{
    context::audio_context::AudioContext,
    generator::SampleGenerator,
    node::{float::Float32Source, saw_oscillator::SawOscillatorNode},
    source::CachedFloatSource,
};

use test_utils::threshold_eq_float32;

#[test]
fn test_saw_oscillator_node_sequence() {
    let oscillator_node = SawOscillatorNode::new(Rc::new(RefCell::new(CachedFloatSource::new(
        Box::new(Float32Source::new(1.)),
    ))));

    let sample_rate = 4.;
    let mut generator = SampleGenerator::new(
        Rc::new(RefCell::new(CachedFloatSource::new(Box::new(
            oscillator_node,
        )))),
        AudioContext::new(sample_rate),
    );

    let num_sets = 100;
    let samples = generator.generate_samples(4 * num_sets + 1);
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
