use std::{
    collections::HashSet,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{anyhow, bail};

use crate::source::Source;

pub type SharedNode = Arc<Mutex<dyn Source>>;
pub type NodeExecutionSchedule = Vec<SharedNode>;

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
    nodes: NodeExecutionSchedule,
    max_id: usize,
) -> Result<NodeExecutionSchedule, anyhow::Error> {
    // Isolated nodes in graphs with multiple nodes have no semantic meaning, so
    // removing them helps verify that the graph is valid
    let trimmed_nodes = remove_isolated_nodes(&nodes);

    // Graph must have exactly one root since our final audio sample comes from the last executed node
    if !is_graph_single_rooted(&trimmed_nodes) {
        bail!("expected a single-rooted graph, instead got a graph with multiple roots")
    }

    let mut schedule = vec![];

    let mut id_to_dependent_ids: Vec<Vec<usize>> = vec![vec![]; max_id + 1];
    let mut id_to_node: Vec<Option<SharedNode>> = vec![None; max_id + 1];

    let mut stack: Vec<usize> = vec![]; // For DFS from leaf nodes
    let mut id_to_num_dependencies_satisfied: Vec<u32> = vec![0; max_id + 1];

    for node in &trimmed_nodes {
        let borrowed_node = node.lock().unwrap();
        id_to_node[borrowed_node.id()] = Some(node.clone());

        let dependency_ids = borrowed_node.dependency_ids();

        for idx in 0..dependency_ids.len() {
            id_to_dependent_ids[dependency_ids[idx]].push(borrowed_node.id());
        }

        // Leaf nodes must be first in the schedule
        if dependency_ids.len() == 0 {
            stack.push(borrowed_node.id());
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

        schedule.push(
            id_to_node[popped_id]
                .clone()
                .ok_or(anyhow!("no node available with id {}", popped_id))?,
        );

        for dependent_id in &id_to_dependent_ids[popped_id] {
            id_to_num_dependencies_satisfied[*dependent_id] += 1;

            // Nodes should only be added to the stack if all their dependencies are satisfied
            if id_to_num_dependencies_satisfied[*dependent_id] as usize
                == id_to_node[*dependent_id]
                    .clone()
                    .ok_or(anyhow!(
                        "no dependent node available with id {}",
                        *dependent_id
                    ))?
                    .lock()
                    .unwrap()
                    .dependency_ids()
                    .len()
            {
                stack.push(*dependent_id);
            }
        }
    }

    if schedule.len() != trimmed_nodes.len() {
        bail!(
            "not all nodes were scheduled. received {} nodes, scheduled only {}",
            trimmed_nodes.len(),
            schedule.len()
        );
    }

    Ok(schedule)
}

pub fn remove_isolated_nodes(nodes: &NodeExecutionSchedule) -> NodeExecutionSchedule {
    let all_nodes = nodes.clone();

    // Graph validation is done sequentially, so locking all nodes for the duration of the function is acceptable
    let isolated_node_ids: HashSet<usize> = {
        let locked_nodes: Vec<MutexGuard<dyn Source>> =
            nodes.iter().map(|node| node.lock().unwrap()).collect();

        locked_nodes
            .iter()
            .map(|node| {
                let num_dependents = locked_nodes
                    .iter()
                    .filter(|other_node| other_node.dependency_ids().contains(&node.id()))
                    .count();
                (node.id(), num_dependents, node.dependency_ids().len())
            })
            .filter(|(_id, num_dependents, num_dependencies)| {
                *num_dependencies == 0 && *num_dependents == 0
            })
            .map(|(id, _num_dependents, _num_dependencies)| id)
            .collect()
    };

    all_nodes
        .into_iter()
        .filter(|node| !isolated_node_ids.contains(&node.lock().unwrap().id()))
        .collect()
}

/// Checks if the audio graph is single-rooted.
/// We want to ensure that all nodes are connected to a single root node.
pub fn is_graph_single_rooted(nodes: &NodeExecutionSchedule) -> bool {
    // Graph validation is done sequentially, so locking all nodes for the duration of the function is acceptable
    let locked_nodes: Vec<MutexGuard<dyn Source>> =
        nodes.iter().map(|node| node.lock().unwrap()).collect();

    let num_roots: usize = locked_nodes
        .iter()
        .map(|node| {
            let num_dependents = locked_nodes
                .iter()
                .filter(|other_node| other_node.dependency_ids().contains(&node.id()))
                .count();
            num_dependents
        })
        .filter(|num_dependents| *num_dependents == 0)
        .count();

    num_roots == 1
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use rand::seq::SliceRandom;

    use crate::{
        context::AudioContext,
        scheduler::{
            NodeExecutionSchedule, build_schedule, is_graph_single_rooted, remove_isolated_nodes,
        },
        source::{NodeOutput, Source},
    };

    #[test]
    fn test_build_schedule_linear() {
        let mut nodes: NodeExecutionSchedule = vec![
            Arc::new(Mutex::new(FloatSource::new(0, 1.))),
            Arc::new(Mutex::new(EchoNode::new(1, 0))),
            Arc::new(Mutex::new(EchoNode::new(2, 1))),
            Arc::new(Mutex::new(EchoNode::new(3, 2))),
        ];

        let mut rng = rand::rng();
        nodes.shuffle(&mut rng);
        let schedule = build_schedule(nodes, 3).unwrap();
        let expected_id_order: Vec<usize> = vec![0, 1, 2, 3];
        for (idx, node) in schedule.iter().enumerate() {
            assert_eq!(
                node.lock().unwrap().id(),
                expected_id_order[idx],
                "expected id {}, got {}",
                expected_id_order[idx],
                node.lock().unwrap().id()
            );
        }
    }

    #[test]
    fn test_build_schedule_branching() {
        let mut nodes: NodeExecutionSchedule = vec![
            Arc::new(Mutex::new(FloatSource::new(0, 1.))),
            Arc::new(Mutex::new(FloatSource::new(1, 1.))),
            Arc::new(Mutex::new(SumNode::new(2, 0, 1))),
            Arc::new(Mutex::new(EchoNode::new(3, 2))),
            Arc::new(Mutex::new(EchoNode::new(4, 3))),
            Arc::new(Mutex::new(SumNode::new(5, 4, 2))),
        ];

        let mut rng = rand::rng();
        nodes.shuffle(&mut rng);

        let schedule = build_schedule(nodes, 5).unwrap();
        let actual_id_order: Vec<usize> = schedule
            .iter()
            .map(|node| node.lock().unwrap().id())
            .collect();

        // There are two valid orders since nodes 0 and 1 don't depend on each other
        let expected_id_order_1: Vec<usize> = vec![0, 1, 2, 3, 4, 5];
        let expected_id_order_2: Vec<usize> = vec![1, 0, 2, 3, 4, 5];

        assert!(
            actual_id_order == expected_id_order_1 || actual_id_order == expected_id_order_2,
            "expected ID order {:?} or {:?}, got {:?}",
            expected_id_order_1,
            expected_id_order_2,
            actual_id_order
        );
    }

    #[test]
    fn test_error_on_cycle() {
        let nodes: NodeExecutionSchedule = vec![
            Arc::new(Mutex::new(FloatSource::new(0, 1.))),
            Arc::new(Mutex::new(SumNode::new(1, 0, 2))),
            Arc::new(Mutex::new(EchoNode::new(2, 1))),
        ];

        let schedule = build_schedule(nodes, 2);
        assert!(schedule.is_err());
    }

    #[test]
    fn test_multiple_roots() {
        let nodes: NodeExecutionSchedule = vec![
            Arc::new(Mutex::new(FloatSource::new(0, 1.))),
            Arc::new(Mutex::new(FloatSource::new(1, 1.))),
        ];

        assert!(!is_graph_single_rooted(&nodes));
    }

    #[test]
    fn test_single_root() {
        let nodes: NodeExecutionSchedule = vec![
            Arc::new(Mutex::new(FloatSource::new(0, 1.))),
            Arc::new(Mutex::new(EchoNode::new(1, 0))),
        ];

        assert!(is_graph_single_rooted(&nodes));
    }

    #[test]
    fn test_remove_isolated_nodes() {
        let nodes: NodeExecutionSchedule = vec![
            Arc::new(Mutex::new(FloatSource::new(0, 1.))),
            Arc::new(Mutex::new(FloatSource::new(1, 1.))),
            Arc::new(Mutex::new(EchoNode::new(2, 0))),
        ];

        let trimmed_nodes = remove_isolated_nodes(&nodes);
        assert_eq!(trimmed_nodes.len(), 2);
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
        fn poll(
            &mut self,
            _audio_context: &AudioContext,
            _id_to_output: &NodeOutput,
        ) -> Option<f32> {
            Some(self.value)
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
        fn poll(
            &mut self,
            _audio_context: &AudioContext,
            id_to_output: &NodeOutput,
        ) -> Option<f32> {
            id_to_output[self.value_source_id]
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
        fn poll(
            &mut self,
            _audio_context: &AudioContext,
            id_to_output: &NodeOutput,
        ) -> Option<f32> {
            id_to_output[self.value_source1_id]
                .zip(id_to_output[self.value_source2_id])
                .map(|(augend, addend)| augend + addend)
        }

        fn id(&self) -> usize {
            self.id
        }

        fn dependency_ids(&self) -> &Vec<usize> {
            &self.dependency_ids
        }
    }
}
