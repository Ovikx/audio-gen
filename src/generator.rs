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
    num_samples: usize,
    schedule: Vec<usize>,
    layered_node_schedule: Vec<Vec<Box<dyn Source>>>,
    layered_output_scratch: Vec<Vec<Vec<Option<f32>>>>, // [layer][node in layer][sample]
    root_id: usize,
}

impl SampleGenerator {
    pub fn new(
        nodes: Vec<Box<dyn Source>>,
        audio_context: AudioContext,
        num_samples: usize,
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
        let mut id_to_node: Vec<Option<Box<dyn Source>>> = (0..=max_id).map(|_| None).collect();

        for node in nodes.into_iter() {
            let id = node.id();
            id_to_node[id] = Some(node);
        }

        let id_to_output = vec![vec![None; num_samples]; max_id + 1];

        // Isolated nodes in graphs with multiple nodes have no semantic meaning, so
        // removing them helps verify that the graph is valid
        let trimmed_references = remove_isolated_references(references);
        let root_id = root_id(&trimmed_references)?;

        let schedule = build_schedule(&trimmed_references, max_id)?;
        let layered_schedule = build_parallel_schedule(&trimmed_references, max_id, root_id)?;

        let mut layered_node_schedule: Vec<Vec<Box<dyn Source>>> = vec![];

        for layer in &layered_schedule {
            layered_node_schedule.push(vec![]);
            for node_id in layer {
                layered_node_schedule
                    .last_mut()
                    .unwrap()
                    .push(id_to_node[*node_id].take().unwrap());
            }
        }

        let layered_output_scratch: Vec<Vec<Vec<Option<f32>>>> = layered_node_schedule
            .iter()
            .map(|layer| layer.iter().map(|_| vec![None; num_samples]).collect())
            .collect();

        Ok(SampleGenerator {
            audio_context: audio_context,
            id_to_output,
            schedule: schedule,
            num_samples,
            root_id,
            layered_node_schedule,
            layered_output_scratch,
        })
    }

    pub fn batch_poll(&mut self) -> Vec<f32> {
        for (layer, scratch_layer) in self
            .layered_node_schedule
            .iter_mut()
            .zip(self.layered_output_scratch.iter_mut())
        {
            layer
                .par_iter_mut()
                .zip(scratch_layer.par_iter_mut())
                .for_each(|(node, out_slice)| {
                    node.batch_poll(
                        self.num_samples,
                        &self.audio_context,
                        &self.id_to_output,
                        out_slice.as_mut_slice(),
                    )
                });

            for (node, out_slice) in layer.iter().zip(scratch_layer.iter_mut()) {
                let id = node.id();
                std::mem::swap(&mut self.id_to_output[id], out_slice);
            }
        }

        self.id_to_output[self.root_id]
            .iter()
            .map(|sample| sample.unwrap_or(0.))
            .collect()
    }
}
