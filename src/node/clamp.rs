use crate::{
    context::audio_context::AudioContext,
    source::{LogicalTimestamp, Source},
};

pub struct ClampNode<S: Source<f32>, L: Source<f32>, U: Source<f32>> {
    sample_source: S,
    lower_bound_source: L,
    upper_bound_source: U,
}

impl<S: Source<f32>, L: Source<f32>, U: Source<f32>> ClampNode<S, L, U> {
    pub fn new(sample_source: S, lower_bound_source: L, upper_bound_source: U) -> Self {
        ClampNode {
            sample_source,
            lower_bound_source,
            upper_bound_source,
        }
    }
}

impl<S: Source<f32>, L: Source<f32>, U: Source<f32>> Source<f32> for ClampNode<S, L, U> {
    fn poll(&mut self, audio_context: &AudioContext, timestamp: LogicalTimestamp) -> Option<f32> {
        let lower_bound = self.lower_bound_source.poll(audio_context, timestamp);
        let upper_bound = self.upper_bound_source.poll(audio_context, timestamp);
        if let (Some(lower_bound), Some(upper_bound)) = (lower_bound, upper_bound) {
            self.sample_source
                .poll(audio_context, timestamp)
                .map(|mut f| {
                    if f > upper_bound {
                        f = upper_bound;
                    }

                    if f < lower_bound {
                        f = lower_bound;
                    }

                    f
                })
        } else {
            None
        }
    }
}
