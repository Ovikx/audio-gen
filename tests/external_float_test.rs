use std::{cell::RefCell, rc::Rc};

use audio_gen::{
    context::AudioContext, generator::SampleGenerator, graph, input_buffer::ExternalInputBuffer,
};

use test_utils::threshold_eq_float32;

#[test]
fn test_external_float_mutation() {
    const INPUT_BUFFER_INDEX: usize = 0;
    const NUM_SAMPLES: usize = 100;

    let mut graph = graph::Graph::new();
    let input_buffer = Rc::new(RefCell::new(ExternalInputBuffer::new(1)));
    graph.insert_external_float_node(input_buffer.clone(), INPUT_BUFFER_INDEX);
    let mut generator = SampleGenerator::new(graph.nodes(), AudioContext::new(1.)).unwrap();

    let samples: Vec<f32> = (0..NUM_SAMPLES)
        .map(|i| {
            input_buffer
                .borrow_mut()
                .update_f32(INPUT_BUFFER_INDEX, i as f32)
                .unwrap();
            generator.poll()
        })
        .collect();
    let expected_samples: Vec<f32> = (0..NUM_SAMPLES).map(|i| i as f32).collect();

    for i in 0..NUM_SAMPLES {
        dbg!(samples[i], expected_samples[i]);
        assert!(threshold_eq_float32(samples[i], expected_samples[i]));
    }
}
