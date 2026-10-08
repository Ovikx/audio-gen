use crate::context::AudioContext;

pub type NodeOutput = Vec<Vec<f32>>;

pub trait Source: Send {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        audio_context: &AudioContext,
        id_to_output: &NodeOutput,
        output: &mut [f32],
    );
    fn id(&self) -> usize; // Stored as a usize since IDs are used for indexing arrays
    fn dependency_ids(&self) -> &Vec<usize>;
}
