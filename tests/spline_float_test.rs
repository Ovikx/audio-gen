use std::{cell::RefCell, rc::Rc};

use audio_gen::{
    context::audio_context::AudioContext,
    generator::SampleGenerator,
    node::{float::Float32Source, spline_float::SplineFloatNode},
    source::CachedFloatSource,
};

use test_utils::threshold_eq_float32;

#[test]
fn test_spline_node_sequence() {
    let spline_node = SplineFloatNode::new(
        Rc::new(RefCell::new(CachedFloatSource::new(Box::new(
            Float32Source::new(1.),
        )))),
        vec![(0.0, 0.0), (1.0, 1.0)],
    );

    let sample_rate = 4.;
    let mut generator = SampleGenerator::new(
        Rc::new(RefCell::new(CachedFloatSource::new(Box::new(spline_node)))),
        AudioContext::new(sample_rate),
    );

    let num_sets = 100;
    let samples = generator.generate_samples(4 * num_sets + 1);
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
