use std::{cell::RefCell, rc::Rc};

use crate::context::audio_context::AudioContext;

pub trait Source<T> {
    fn poll(&mut self, audio_context: &AudioContext) -> Option<T>;
}

pub type SharedFloatSource = Rc<RefCell<Box<dyn Source<f32>>>>;
