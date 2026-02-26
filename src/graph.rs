use std::sync::{Arc, Mutex};

use crate::{
    input_buffer::SharedExternalInputBuffer,
    math::spline_polynomial::Point,
    node::{
        AbsoluteValue, ExternalFloatNode, FloatSource, MultiplyNode, NoiseNode, SawOscillatorNode,
        SequenceNode, SineOscillatorNode, SourceInterval, SplineFloatNode, SquareOscillatorNode,
        SumNode,
    },
    scheduler::SharedNode,
};

pub struct Graph {
    current_id: usize,
    nodes: Vec<SharedNode>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            current_id: 0,
            nodes: vec![],
        }
    }

    pub fn float_node(&mut self, value: f32) -> usize {
        let id = self.current_id;
        self.nodes
            .push(Arc::new(Mutex::new(FloatSource::new(id, value))));
        self.current_id += 1;
        id
    }

    pub fn external_float_node(
        &mut self,
        input_buffer: SharedExternalInputBuffer,
        input_buffer_index: usize,
    ) -> usize {
        let id = self.current_id;
        self.nodes.push(Arc::new(Mutex::new(ExternalFloatNode::new(
            id,
            input_buffer,
            input_buffer_index,
        ))));
        self.current_id += 1;
        id
    }

    pub fn multiply_node(
        &mut self,
        multiplicand_source_id: usize,
        multiplier_source_id: usize,
    ) -> usize {
        let id = self.current_id;
        self.nodes.push(Arc::new(Mutex::new(MultiplyNode::new(
            id,
            multiplicand_source_id,
            multiplier_source_id,
        ))));
        self.current_id += 1;
        id
    }

    pub fn saw_oscillator_node(&mut self, frequency_source_id: usize) -> usize {
        let id = self.current_id;
        self.nodes.push(Arc::new(Mutex::new(SawOscillatorNode::new(
            id,
            frequency_source_id,
        ))));
        self.current_id += 1;
        id
    }

    pub fn sine_oscillator_node(&mut self, frequency_source_id: usize) -> usize {
        let id = self.current_id;
        self.nodes.push(Arc::new(Mutex::new(SineOscillatorNode::new(
            id,
            frequency_source_id,
        ))));
        self.current_id += 1;
        id
    }

    pub fn square_oscillator_node(&mut self, frequency_source_id: usize) -> usize {
        let id = self.current_id;
        self.nodes
            .push(Arc::new(Mutex::new(SquareOscillatorNode::new(
                id,
                frequency_source_id,
            ))));
        self.current_id += 1;
        id
    }

    pub fn spline_float_node(&mut self, frequency_source_id: usize, points: Vec<Point>) -> usize {
        let id = self.current_id;
        self.nodes.push(Arc::new(Mutex::new(SplineFloatNode::new(
            id,
            frequency_source_id,
            points,
        ))));
        self.current_id += 1;
        id
    }

    pub fn sum_node(&mut self, augend_source_id: usize, addend_source_id: usize) -> usize {
        let id = self.current_id;
        self.nodes.push(Arc::new(Mutex::new(SumNode::new(
            id,
            augend_source_id,
            addend_source_id,
        ))));
        self.current_id += 1;
        id
    }

    pub fn noise_node(&mut self, seed: u64) -> usize {
        let id = self.current_id;
        self.nodes
            .push(Arc::new(Mutex::new(NoiseNode::new(id, seed))));
        self.current_id += 1;
        id
    }

    pub fn absolute_value_node(&mut self, source_id: usize) -> usize {
        let id = self.current_id;
        self.nodes
            .push(Arc::new(Mutex::new(AbsoluteValue::new(id, source_id))));
        self.current_id += 1;
        id
    }

    pub fn sequence_node(&mut self, source_intervals: Vec<SourceInterval>) -> usize {
        let id = self.current_id;
        self.nodes.push(Arc::new(Mutex::new(SequenceNode::new(
            id,
            source_intervals,
        ))));
        self.current_id += 1;
        id
    }

    pub fn nodes(&self) -> Vec<SharedNode> {
        self.nodes.clone()
    }
}
