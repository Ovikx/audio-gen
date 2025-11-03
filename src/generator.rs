use std::{cell::RefCell, rc::Rc};

use crate::{context::audio_context::AudioContext, node::source::Source};

pub struct SampleGenerator {
    output_source: Rc<RefCell<Box<dyn Source<f32>>>>,
    audio_context: AudioContext,
}

impl SampleGenerator {
    pub fn new(
        output_source: Rc<RefCell<Box<dyn Source<f32>>>>,
        audio_context: AudioContext,
    ) -> SampleGenerator {
        SampleGenerator {
            output_source,
            audio_context,
        }
    }

    pub fn generate_samples(&mut self, num_samples: u32) -> Vec<f32> {
        let mut samples = vec![];
        for _ in 0..num_samples {
            samples.push(
                self.output_source
                    .borrow_mut()
                    .poll(&self.audio_context)
                    .unwrap_or(0.0),
            );
        }
        samples
    }
}
