use crate::{
    context::audio_context::AudioContext,
    source::{LogicalTimestamp, SharedCachedFloatSource},
};

pub struct SampleGenerator {
    output_source: SharedCachedFloatSource,
    audio_context: AudioContext,
    logical_clock: LogicalTimestamp,
}

impl SampleGenerator {
    pub fn new(
        output_source: SharedCachedFloatSource,
        audio_context: AudioContext,
    ) -> SampleGenerator {
        SampleGenerator {
            output_source,
            audio_context,
            logical_clock: 0,
        }
    }

    pub fn generate_samples(&mut self, num_samples: u32) -> Vec<f32> {
        let mut samples = vec![];
        for _ in 0..num_samples {
            samples.push(
                self.output_source
                    .borrow_mut()
                    .poll(&self.audio_context, self.logical_clock)
                    .unwrap_or(0.0),
            );
            self.logical_clock = self.logical_clock.wrapping_add(1);
        }
        samples
    }
}
