use std::sync::{Arc, Mutex};

use crate::{sequencer::Sequencer, source::Source};

/// Wrapper for a Sequencer.
/// Nodes cannot use samples directly from a Sequencer since it doesn't
/// implement `Source`. This node makes it possible to do so.
pub struct SequenceNode {
    id: usize,
    sequencer: Sequencer,
    dependency_ids: Vec<usize>,
}

impl SequenceNode {
    pub fn new(id: usize, sequencer: Sequencer) -> Self {
        SequenceNode {
            id,
            sequencer,
            dependency_ids: vec![],
        }
    }
}

impl Source for SequenceNode {
    fn poll(
        &mut self,
        _audio_context: &crate::context::AudioContext,
        _id_to_output: &crate::source::NodeOutput,
    ) -> Option<f32> {
        self.sequencer.poll()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
