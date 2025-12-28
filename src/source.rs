use crate::context::audio_context::AudioContext;

pub type NodeOutput = Vec<Option<f32>>;

pub trait Source {
    fn poll(&mut self, audio_context: &AudioContext, id_to_output: &NodeOutput) -> Option<f32>;
    fn id(&self) -> usize; // Stored as a usize since IDs are used for indexing arrays
    fn dependency_ids(&self) -> &Vec<usize>;
}
