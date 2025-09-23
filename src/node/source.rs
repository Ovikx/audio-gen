use crate::context::audio_context::AudioContext;

pub trait Source<T> {
    fn poll(&mut self, audio_context: &AudioContext) -> Option<T>;
}
