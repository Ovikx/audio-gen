use std::{cell::RefCell, io::Error, rc::Rc};

use crate::source::Source;

pub fn build_schedule(
    nodes: Vec<Rc<RefCell<dyn Source>>>,
    max_id: usize,
) -> Result<Vec<Rc<RefCell<dyn Source>>>, Error> {
    dbg!("max_id ", max_id);
    let mut schedule = vec![];

    let mut id_to_dependent_ids: Vec<Vec<usize>> = vec![vec![]; max_id + 1];
    let mut id_to_node: Vec<Option<Rc<RefCell<dyn Source>>>> = vec![None; max_id + 1];

    let mut stack: Vec<usize> = vec![]; // For DFS from leaf nodes
    let mut id_to_num_dependencies_satisfied: Vec<u32> = vec![0; max_id + 1];

    for node in nodes {
        let borrowed_node = node.borrow();
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
        return Err(Error::new(
            std::io::ErrorKind::Other,
            "graph must have leaves as a starting point for scheduling",
        ));
    }

    while stack.len() > 0 {
        let popped_id = stack.pop().ok_or(Error::new(
            std::io::ErrorKind::Other,
            "attempted to pop from empty stack",
        ))?;

        schedule.push(id_to_node[popped_id].clone().ok_or(Error::new(
            std::io::ErrorKind::Other,
            format!("no node available with id {}", popped_id),
        ))?);

        for dependent_id in &id_to_dependent_ids[popped_id] {
            id_to_num_dependencies_satisfied[*dependent_id] += 1;

            // Nodes should only be added to the stack if all their dependencies are satisfied
            if id_to_num_dependencies_satisfied[*dependent_id] as usize
                == id_to_node[*dependent_id]
                    .clone()
                    .ok_or(Error::new(
                        std::io::ErrorKind::Other,
                        format!("no dependent node available with id {}", *dependent_id),
                    ))?
                    .borrow()
                    .dependency_ids()
                    .len()
            {
                stack.push(*dependent_id);
            }
        }
    }

    Ok(schedule)
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use rand::seq::SliceRandom;

    use crate::{
        context::audio_context::AudioContext,
        generator::scheduler::build_schedule,
        source::{NodeOutput, Source},
    };

    #[test]
    fn test_build_schedule_linear() {
        let mut nodes: Vec<Rc<RefCell<dyn Source>>> = vec![
            Rc::new(RefCell::new(FloatSource::new(0, 1.))),
            Rc::new(RefCell::new(EchoNode::new(1, 0))),
            Rc::new(RefCell::new(EchoNode::new(2, 1))),
            Rc::new(RefCell::new(EchoNode::new(3, 2))),
        ];

        let mut rng = rand::rng();
        nodes.shuffle(&mut rng);
        let schedule = build_schedule(nodes, 3).unwrap();
        let expected_id_order: Vec<usize> = vec![0, 1, 2, 3];
        for (idx, node) in schedule.iter().enumerate() {
            assert_eq!(
                node.borrow().id(),
                expected_id_order[idx],
                "expected id {}, got {}",
                expected_id_order[idx],
                node.borrow().id()
            );
        }
    }

    #[test]
    fn test_build_schedule_branching() {
        let mut nodes: Vec<Rc<RefCell<dyn Source>>> = vec![
            Rc::new(RefCell::new(FloatSource::new(0, 1.))),
            Rc::new(RefCell::new(FloatSource::new(1, 1.))),
            Rc::new(RefCell::new(SumNode::new(2, 0, 1))),
            Rc::new(RefCell::new(EchoNode::new(3, 2))),
            Rc::new(RefCell::new(EchoNode::new(4, 3))),
            Rc::new(RefCell::new(SumNode::new(5, 4, 2))),
        ];

        let mut rng = rand::rng();
        nodes.shuffle(&mut rng);

        let schedule = build_schedule(nodes, 5).unwrap();
        let actual_id_order: Vec<usize> = schedule.iter().map(|node| node.borrow().id()).collect();

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
