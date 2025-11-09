use crate::context::audio_context::AudioContext;
use std::{cell::RefCell, rc::Rc};

pub trait Source<T> {
    fn poll(&mut self, audio_context: &AudioContext, timestamp: LogicalTimestamp) -> Option<T>;
}

pub struct CachedFloatSource {
    sample_source: Box<dyn Source<f32>>,
    curr_timestamp: LogicalTimestamp,
    cached_sample: Option<f32>,
}

impl CachedFloatSource {
    pub fn new(sample_source: Box<dyn Source<f32>>) -> Self {
        CachedFloatSource {
            sample_source,
            curr_timestamp: 0,
            cached_sample: None,
        }
    }

    pub fn poll(
        &mut self,
        audio_context: &AudioContext,
        timestamp: LogicalTimestamp,
    ) -> Option<f32> {
        if timestamp != self.curr_timestamp {
            self.cached_sample = self.sample_source.poll(audio_context, timestamp);
            self.curr_timestamp = timestamp;
        }

        self.cached_sample
    }
}

pub type LogicalTimestamp = u8;
pub type SharedCachedFloatSource = Rc<RefCell<CachedFloatSource>>;
