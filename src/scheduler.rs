use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, bail};

use crate::source::Source;

pub type SharedNode = Arc<Mutex<dyn Source>>;
pub type NodeExecutionSchedule = Vec<Box<dyn Source>>;

/// Builds a node execution schedule defining the order nodes are polled.
///
/// This function assumes that the input audio graph is messy, so it performs
/// the following validations and transformations to ensure the audio graph is
/// schedulable:
///
/// - In a graph with multiple nodes, isolated nodes (i.e. those with no
///   dependencies or dependents) are removed
/// - Scheduling fails if there are multiple graph components
pub fn build_schedule(
    references: &Vec<Reference>,
    max_id: usize,
) -> Result<Vec<usize>, anyhow::Error> {
    // Graph must have exactly one root since our final audio sample comes from the last executed node
    if !is_graph_single_rooted(&references) {
        bail!("expected a single-rooted graph, instead got a graph with multiple roots")
    }

    let mut schedule = vec![];

    let mut id_to_dependent_ids: Vec<Vec<usize>> = vec![vec![]; max_id + 1];
    let mut id_to_reference: Vec<Option<&Reference>> = vec![None; max_id + 1];

    let mut stack: Vec<usize> = vec![]; // For DFS from leaf nodes
    let mut id_to_num_dependencies_satisfied: Vec<u32> = vec![0; max_id + 1];

    for reference in references {
        let dependency_ids = &reference.child_ids;
        id_to_reference[reference.id] = Some(reference);

        for idx in 0..dependency_ids.len() {
            id_to_dependent_ids[dependency_ids[idx]].push(reference.id);
        }

        // Leaf nodes must be first in the schedule
        if dependency_ids.len() == 0 {
            stack.push(reference.id);
        }
    }

    // Graph must have leaves
    if stack.len() == 0 {
        bail!("graph must have leaves as a starting point for scheduling")
    }

    while stack.len() > 0 {
        let popped_id = stack
            .pop()
            .ok_or(anyhow!("attempted to pop from empty stack"))?;

        schedule.push(popped_id);

        for dependent_id in &id_to_dependent_ids[popped_id] {
            id_to_num_dependencies_satisfied[*dependent_id] += 1;

            // Nodes should only be added to the stack if all their dependencies are satisfied
            if id_to_num_dependencies_satisfied[*dependent_id] as usize
                == id_to_reference[*dependent_id]
                    .clone()
                    .ok_or(anyhow!(
                        "no dependent node available with id {}",
                        *dependent_id
                    ))?
                    .child_ids
                    .len()
            {
                stack.push(*dependent_id);
            }
        }
    }

    if schedule.len() != references.len() {
        bail!(
            "not all nodes were scheduled. received {} nodes, scheduled only {}",
            references.len(),
            schedule.len()
        );
    }

    Ok(schedule)
}

pub fn remove_isolated_references(references: Vec<Reference>) -> Vec<Reference> {
    // Graph validation is done sequentially, so locking all nodes for the duration of the function is acceptable
    let isolated_node_ids: HashSet<usize> = references
        .iter()
        .map(|reference| {
            let num_dependents = references
                .iter()
                .filter(|other_node| other_node.child_ids.contains(&reference.id))
                .count();
            (reference.id, num_dependents, reference.child_ids.len())
        })
        .filter(|(_id, num_dependents, num_dependencies)| {
            *num_dependencies == 0 && *num_dependents == 0
        })
        .map(|(id, _num_dependents, _num_dependencies)| id)
        .collect();

    references
        .into_iter()
        .filter(|reference| !isolated_node_ids.contains(&reference.id))
        .collect()
}

/// Checks if the audio graph is single-rooted.
/// We want to ensure that all nodes are connected to a single root node.
pub fn is_graph_single_rooted(references: &Vec<Reference>) -> bool {
    let num_roots: usize = references
        .iter()
        .map(|node| {
            let num_dependents = references
                .iter()
                .filter(|other_node| other_node.child_ids.contains(&node.id))
                .count();
            num_dependents
        })
        .filter(|num_dependents| *num_dependents == 0)
        .count();

    num_roots == 1
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub id: usize,
    pub child_ids: Vec<usize>,
}

pub fn nodes_to_references(nodes: &Vec<Box<dyn Source>>) -> Result<Vec<Reference>, anyhow::Error> {
    let mut id_to_reference: HashMap<usize, Reference> = HashMap::new();

    for node in nodes {
        if !id_to_reference.contains_key(&node.id()) {
            id_to_reference.insert(
                node.id(),
                Reference {
                    id: node.id(),
                    child_ids: vec![],
                },
            );
        }

        let reference = id_to_reference
            .get_mut(&node.id())
            .ok_or_else(|| anyhow!("reference with id {} does not exist in map", node.id()))?;

        for dependency_id in node.dependency_ids() {
            reference.child_ids.push(*dependency_id);
        }
    }

    Ok(id_to_reference.into_values().collect())
}

pub fn root_id(references: &Vec<Reference>) -> Result<usize, anyhow::Error> {
    let mut id_to_num_dependents: HashMap<usize, u32> =
        references.iter().map(|node| (node.id, 0)).collect();

    for reference in references {
        for child_id in &reference.child_ids {
            let current_value = id_to_num_dependents
                .get(&child_id)
                .ok_or_else(|| anyhow!("no reference with id {} exists", child_id))?;
            id_to_num_dependents.insert(*child_id, current_value + 1);
        }
    }

    for (id, num_dependents) in id_to_num_dependents {
        if num_dependents == 0 {
            return Ok(id);
        }
    }

    bail!("expected non-empty graph");
}

pub fn reversed_references(
    references: &Vec<Reference>,
    root_id: usize,
) -> Result<HashMap<usize, Reference>, anyhow::Error> {
    let mut id_to_reference: HashMap<usize, Reference> = HashMap::new();

    // Root node isn't a child of any other node, so its reference won't be created unless we do it manually
    id_to_reference.insert(
        root_id,
        Reference {
            id: root_id,
            child_ids: vec![],
        },
    );

    for reference in references {
        for child_id in &reference.child_ids {
            if !id_to_reference.contains_key(&child_id) {
                id_to_reference.insert(
                    *child_id,
                    Reference {
                        id: *child_id,
                        child_ids: vec![],
                    },
                );
            }

            let child_reference = id_to_reference
                .get_mut(&child_id)
                .ok_or_else(|| anyhow!("reference with id {} does not exist in map", child_id))?;
            child_reference.child_ids.push(reference.id);
        }
    }
    Ok(id_to_reference)
}

pub type LayeredSchedule = Vec<Vec<usize>>;

pub fn build_parallel_schedule(
    references: &Vec<Reference>,
    max_id: usize,
    root_id: usize,
) -> Result<LayeredSchedule, anyhow::Error> {
    // Graph must have exactly one root since our final audio sample comes from the last executed node
    if !is_graph_single_rooted(&references) {
        bail!("expected a single-rooted graph, instead got a graph with multiple roots")
    }

    let reversed_references = reversed_references(&references, root_id)?;
    let mut id_to_reference: Vec<Option<&Reference>> = vec![None; max_id + 1];
    let mut id_to_depth: Vec<usize> = vec![0; max_id + 1];
    for reference in references {
        id_to_reference[reference.id] = Some(reference);
    }

    // Start with leaf nodes since all depths can be derived
    // Vector of (id, depth)
    let mut stack: Vec<(usize, usize)> = references
        .iter()
        .filter(|reference| reference.child_ids.len() == 0)
        .map(|reference| (reference.id, 0))
        .collect();

    while stack.len() > 0 {
        let (reference_id, depth) = stack
            .pop()
            .ok_or_else(|| anyhow!("attempted to pop from empty stack"))?;

        let dependent_ids = &reversed_references
            .get(&reference_id)
            .ok_or_else(|| anyhow!("no reference with id {} exists", reference_id))?
            .child_ids;

        for dependent_id in dependent_ids {
            id_to_depth[*dependent_id] = max(id_to_depth[*dependent_id], depth + 1);
            stack.push((*dependent_id, depth + 1));
        }
    }

    let max_depth = *id_to_depth
        .iter()
        .max()
        .ok_or_else(|| anyhow!("expected non-empty graph"))?;

    let mut schedule: LayeredSchedule = vec![vec![]; max_depth + 1];
    for (id, depth) in id_to_depth.iter().enumerate() {
        if let Some(reference) = id_to_reference[id].clone() {
            schedule[*depth].push(reference.id);
        }
    }
    Ok(schedule)
}
#[cfg(test)]
mod tests {
    use rand::seq::SliceRandom;

    use crate::{
        context::AudioContext,
        scheduler::{
            NodeExecutionSchedule, build_parallel_schedule, build_schedule, is_graph_single_rooted,
            nodes_to_references, remove_isolated_references, root_id,
        },
        source::{NodeOutput, Source},
    };

    #[test]
    fn test_build_schedule_linear() {
        let mut nodes: NodeExecutionSchedule = vec![
            Box::new(FloatSource::new(0, 1.)),
            Box::new(EchoNode::new(1, 0)),
            Box::new(EchoNode::new(2, 1)),
            Box::new(EchoNode::new(3, 2)),
        ];

        let mut rng = rand::rng();
        nodes.shuffle(&mut rng);
        let references = nodes_to_references(&nodes).unwrap();
        let schedule = build_schedule(&references, 3).unwrap();
        let expected_id_order: Vec<usize> = vec![0, 1, 2, 3];
        assert_eq!(schedule, expected_id_order);
        // for (idx, node) in schedule.iter().enumerate() {
        //     assert_eq!(
        //         node.lock().unwrap().id(),
        //         expected_id_order[idx],
        //         "expected id {}, got {}",
        //         expected_id_order[idx],
        //         node.lock().unwrap().id()
        //     );
        // }
    }

    #[test]
    fn test_build_schedule_branching() {
        let mut nodes: NodeExecutionSchedule = vec![
            Box::new(FloatSource::new(0, 1.)),
            Box::new(FloatSource::new(1, 1.)),
            Box::new(SumNode::new(2, 0, 1)),
            Box::new(EchoNode::new(3, 2)),
            Box::new(EchoNode::new(4, 3)),
            Box::new(SumNode::new(5, 4, 2)),
        ];

        let mut rng = rand::rng();
        nodes.shuffle(&mut rng);

        let references = nodes_to_references(&nodes).unwrap();
        let schedule = build_schedule(&references, 5).unwrap();

        // There are two valid orders since nodes 0 and 1 don't depend on each other
        let expected_id_order_1: Vec<usize> = vec![0, 1, 2, 3, 4, 5];
        let expected_id_order_2: Vec<usize> = vec![1, 0, 2, 3, 4, 5];

        assert!(
            schedule == expected_id_order_1 || schedule == expected_id_order_2,
            "expected ID order {:?} or {:?}, got {:?}",
            expected_id_order_1,
            expected_id_order_2,
            schedule
        );
    }

    #[test]
    fn test_error_on_cycle() {
        let nodes: NodeExecutionSchedule = vec![
            Box::new(FloatSource::new(0, 1.)),
            Box::new(SumNode::new(1, 0, 2)),
            Box::new(EchoNode::new(2, 1)),
        ];

        let references = nodes_to_references(&nodes).unwrap();
        let schedule = build_schedule(&references, 2);
        assert!(schedule.is_err());
    }

    #[test]
    fn test_multiple_roots() {
        let nodes: NodeExecutionSchedule = vec![
            Box::new(FloatSource::new(0, 1.)),
            Box::new(FloatSource::new(1, 1.)),
        ];

        let references = nodes_to_references(&nodes).unwrap();
        assert!(!is_graph_single_rooted(&references));
    }

    #[test]
    fn test_single_root() {
        let nodes: NodeExecutionSchedule = vec![
            Box::new(FloatSource::new(0, 1.)),
            Box::new(EchoNode::new(1, 0)),
        ];

        let references = nodes_to_references(&nodes).unwrap();
        assert!(is_graph_single_rooted(&references));
    }

    #[test]
    fn test_remove_isolated_nodes() {
        let nodes: NodeExecutionSchedule = vec![
            Box::new(FloatSource::new(0, 1.)),
            Box::new(FloatSource::new(1, 1.)),
            Box::new(EchoNode::new(2, 0)),
        ];

        let references = nodes_to_references(&nodes).unwrap();
        let trimmed_nodes = remove_isolated_references(references);
        assert_eq!(trimmed_nodes.len(), 2);
    }

    #[test]
    fn test_build_parallel_schedule() {
        let nodes: NodeExecutionSchedule = vec![
            Box::new(FloatSource::new(0, 1.)),
            Box::new(EchoNode::new(1, 0)),
            Box::new(EchoNode::new(2, 1)),
            Box::new(EchoNode::new(3, 2)),
        ];

        let references = nodes_to_references(&nodes).unwrap();
        let root_id = root_id(&references).unwrap();
        let schedule = build_parallel_schedule(&references, 3, root_id).unwrap();
        assert!(schedule.len() == 4);
        for layer in schedule {
            assert!(layer.len() == 1)
        }
    }

    #[test]
    fn test_build_parallel_schedule_branching() {
        let mut nodes: NodeExecutionSchedule = vec![
            Box::new(FloatSource::new(0, 1.)),
            Box::new(FloatSource::new(1, 1.)),
            Box::new(SumNode::new(2, 0, 1)),
            Box::new(EchoNode::new(3, 2)),
            Box::new(EchoNode::new(4, 3)),
            Box::new(SumNode::new(5, 4, 2)),
        ];

        let mut rng = rand::rng();
        nodes.shuffle(&mut rng);

        let references = nodes_to_references(&nodes).unwrap();
        let root_id = root_id(&references).unwrap();
        let schedule = build_parallel_schedule(&references, 5, root_id).unwrap();

        assert!(schedule.len() == 5);
    }

    #[test]
    fn test_build_parallel_schedule_join() {
        let mut nodes: NodeExecutionSchedule = vec![
            Box::new(FloatSource::new(0, 1.)),
            Box::new(FloatSource::new(1, 1.)),
            Box::new(SumNode::new(2, 0, 1)),
            Box::new(FloatSource::new(3, 1.)),
            Box::new(FloatSource::new(4, 1.)),
            Box::new(SumNode::new(5, 3, 4)),
            Box::new(SumNode::new(6, 5, 2)),
        ];

        let mut rng = rand::rng();
        nodes.shuffle(&mut rng);

        let references = nodes_to_references(&nodes).unwrap();
        let root_id = root_id(&references).unwrap();
        let schedule = build_parallel_schedule(&references, 6, root_id).unwrap();

        assert!(schedule.len() == 3);
    }

    pub struct FloatSource {
        id: usize,
        value: f32,
        dependency_ids: Vec<usize>,
    }

    impl FloatSource {
        pub fn new(id: usize, value: f32) -> Self {
            FloatSource {
                id,
                value,
                dependency_ids: vec![],
            }
        }
    }

    impl Source for FloatSource {
        fn batch_poll(
            &mut self,
            num_samples: usize,
            _audio_context: &AudioContext,
            _id_to_output: &NodeOutput,
            output: &mut [Option<f32>],
        ) {
            for idx in 0..num_samples {
                output[idx] = Some(self.value);
            }
        }

        fn id(&self) -> usize {
            self.id
        }

        fn dependency_ids(&self) -> &Vec<usize> {
            &self.dependency_ids
        }
    }

    pub struct EchoNode {
        id: usize,
        value_source_id: usize,
        dependency_ids: Vec<usize>,
    }

    impl EchoNode {
        pub fn new(id: usize, value_source_id: usize) -> Self {
            EchoNode {
                id,
                value_source_id,
                dependency_ids: vec![value_source_id],
            }
        }
    }

    impl Source for EchoNode {
        fn batch_poll(
            &mut self,
            num_samples: usize,
            _audio_context: &AudioContext,
            id_to_output: &NodeOutput,
            output: &mut [Option<f32>],
        ) {
            for idx in 0..num_samples {
                output[idx] = id_to_output[self.value_source_id][idx];
            }
        }

        fn id(&self) -> usize {
            self.id
        }

        fn dependency_ids(&self) -> &Vec<usize> {
            &self.dependency_ids
        }
    }

    pub struct SumNode {
        id: usize,
        value_source1_id: usize,
        value_source2_id: usize,
        dependency_ids: Vec<usize>,
    }

    impl SumNode {
        pub fn new(id: usize, value_source1_id: usize, value_source2_id: usize) -> Self {
            SumNode {
                id,
                value_source1_id: value_source1_id,
                value_source2_id: value_source2_id,
                dependency_ids: vec![value_source1_id, value_source2_id],
            }
        }
    }

    impl Source for SumNode {
        fn batch_poll(
            &mut self,
            num_samples: usize,
            _audio_context: &AudioContext,
            id_to_output: &NodeOutput,
            output: &mut [Option<f32>],
        ) {
            for idx in 0..num_samples {
                output[idx] = id_to_output[self.value_source1_id][idx]
                    .zip(id_to_output[self.value_source2_id][idx])
                    .map(|(augend, addend)| augend + addend);
            }
        }

        fn id(&self) -> usize {
            self.id
        }

        fn dependency_ids(&self) -> &Vec<usize> {
            &self.dependency_ids
        }
    }
}
