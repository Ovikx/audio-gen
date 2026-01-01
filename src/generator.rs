use std::io::Error;

use crate::{
    context::AudioContext,
    scheduler::{NodeExecutionSchedule, build_schedule},
    source::NodeOutput,
};

pub struct SampleGenerator {
    audio_context: AudioContext,
    id_to_output: NodeOutput,
    schedule: NodeExecutionSchedule,
}

impl SampleGenerator {
    pub fn new(nodes: NodeExecutionSchedule, audio_context: AudioContext) -> Result<Self, Error> {
        // TODO: We need a renaming pass before we do anything; we should try to make the vectors as small as possible. There might be a case where a user assigns a node an ID of 1<<31 or something
        let max_id: usize = nodes
            .iter()
            .map(|node| node.borrow().id())
            .max()
            .ok_or(Error::new(std::io::ErrorKind::Other, "empty node vector"))?;

        let id_to_output = vec![None; max_id + 1];

        let schedule = build_schedule(nodes, max_id)?;
        Ok(SampleGenerator {
            audio_context: audio_context,
            id_to_output: id_to_output,
            schedule: schedule,
        })
    }

    pub fn batch_poll(&mut self, num_samples: u32) -> Vec<f32> {
        let mut samples = vec![];
        for _ in 0..num_samples {
            for node in &self.schedule {
                let mut borrowed_node = node.borrow_mut();
                self.id_to_output[borrowed_node.id()] =
                    borrowed_node.poll(&self.audio_context, &self.id_to_output);
            }

            let root_sample =
                self.id_to_output[self.schedule[self.schedule.len() - 1].borrow().id()];
            samples.push(root_sample.unwrap_or(0.));
        }
        samples
    }
}
