use rayon::prelude::*;
use std::sync::MutexGuard;

use crate::{
    context::AudioContext,
    node::FloatSource,
    scheduler::{
        LayeredSchedule, build_parallel_schedule, build_schedule, nodes_to_references,
        remove_isolated_references, root_id,
    },
    source::{NodeOutput, Source},
};

use anyhow::anyhow;

pub struct SampleGenerator {
    audio_context: AudioContext,
    id_to_output: NodeOutput,
    id_to_node: Vec<Box<dyn Source>>,
    schedule: Vec<usize>,
    layered_schedule: LayeredSchedule,
    root_id: usize,
}

impl SampleGenerator {
    pub fn new(
        nodes: Vec<Box<dyn Source>>,
        audio_context: AudioContext,
    ) -> Result<Self, anyhow::Error> {
        // TODO: We need a renaming pass before we do anything; we should try to make the vectors as small as possible. There might be a case where a user assigns a node an ID of 1<<31 or something
        let references = nodes_to_references(&nodes)?;
        let max_id: usize = references
            .iter()
            .map(|reference| reference.id)
            .max()
            .ok_or(anyhow!("empty node vector"))?;

        // We use a default node for cases where a node with a particular ID instead of using Option<_>.
        // This is to avoid having to unwrap the Option. We set the default node's ID to an unreachable value.
        // Default nodes are not touched in normal cases.
        let mut id_to_node: Vec<Box<dyn Source>> = (0..=max_id)
            .map(|_| Box::new(FloatSource::new(max_id + 1, 0.)) as Box<dyn Source>)
            .collect();

        for node in nodes.into_iter() {
            let id = node.id();
            id_to_node[id] = node;
        }

        let id_to_output = vec![None; max_id + 1];

        // Isolated nodes in graphs with multiple nodes have no semantic meaning, so
        // removing them helps verify that the graph is valid
        let trimmed_references = remove_isolated_references(references);
        let root_id = root_id(&trimmed_references)?;

        let schedule = build_schedule(&trimmed_references, max_id)?;
        let layered_schedule = build_parallel_schedule(&trimmed_references, max_id, root_id)?;
        Ok(SampleGenerator {
            audio_context: audio_context,
            id_to_output: id_to_output,
            schedule: schedule,
            layered_schedule,
            root_id,
            id_to_node,
        })
    }

    pub fn poll(&mut self) -> f32 {
        for node_id in &self.schedule {
            let node = &mut self.id_to_node[*node_id];
            self.id_to_output[node.id()] = node.poll(&self.audio_context, &self.id_to_output);
        }

        let root_sample = self.id_to_output[self.schedule[self.schedule.len() - 1]];
        root_sample.unwrap_or(0.)
    }

    pub fn batch_poll(&mut self, num_samples: u32) -> Vec<f32> {
        let mut samples = vec![];
        for _ in 0..num_samples {
            for node_id in &self.schedule {
                let node = &mut self.id_to_node[*node_id];
                self.id_to_output[node.id()] = node.poll(&self.audio_context, &self.id_to_output);
            }

            let root_sample = self.id_to_output[self.schedule[self.schedule.len() - 1]];
            samples.push(root_sample.unwrap_or(0.));
        }

        samples
    }

    // pub fn batch_poll2(&mut self, num_samples: u32) -> Vec<f32> {
    //     let mut samples: Vec<f32> = vec![];
    //     for _ in 0..num_samples {
    //         for layer in &self.layered_schedule {
    //             let results: Vec<(usize, Option<f32>)> = layer
    //                 .par_iter()
    //                 .map(|node| {
    //                     let mut guarded_node = node.lock().unwrap();
    //                     let sample = guarded_node.poll(&self.audio_context, &self.id_to_output);
    //                     (guarded_node.id(), sample)
    //                 })
    //                 .collect();

    //             for (id, sample) in results {
    //                 self.id_to_output[id] = sample;
    //             }
    //         }
    //         samples.push(self.id_to_output[self.root_id].unwrap_or(0.0));
    //     }

    //     samples
    // }
}
