use std::{cell::RefCell, rc::Rc};

use crate::{
    input_buffer::SharedExternalInputBuffer,
    math::spline_polynomial::Point,
    node::{
        ExternalFloatNode, FloatSource, MultiplyNode, SawOscillatorNode, SineOscillatorNode,
        SplineFloatNode, SquareOscillatorNode, SumNode,
    },
    source::Source,
};

pub trait SerializableNode {}

pub struct Graph {
    current_id: usize,
    nodes: Vec<Rc<RefCell<dyn Source>>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            current_id: 0,
            nodes: vec![],
        }
    }

    pub fn insert_float_node(&mut self, value: f32) -> usize {
        let id = self.current_id;
        self.nodes
            .push(Rc::new(RefCell::new(FloatSource::new(id, value))));
        self.current_id += 1;
        id
    }

    pub fn insert_external_float_node(
        &mut self,
        input_buffer: SharedExternalInputBuffer,
        input_buffer_index: usize,
    ) -> usize {
        let id = self.current_id;
        self.nodes.push(Rc::new(RefCell::new(ExternalFloatNode::new(
            id,
            input_buffer,
            input_buffer_index,
        ))));
        self.current_id += 1;
        id
    }

    pub fn insert_multiply_node(
        &mut self,
        multiplicand_source_id: usize,
        multiplier_source_id: usize,
    ) -> usize {
        let id = self.current_id;
        self.nodes.push(Rc::new(RefCell::new(MultiplyNode::new(
            id,
            multiplicand_source_id,
            multiplier_source_id,
        ))));
        self.current_id += 1;
        id
    }

    pub fn insert_saw_oscillator_node(&mut self, frequency_source_id: usize) -> usize {
        let id = self.current_id;
        self.nodes.push(Rc::new(RefCell::new(SawOscillatorNode::new(
            id,
            frequency_source_id,
        ))));
        self.current_id += 1;
        id
    }

    pub fn insert_sine_oscillator_node(&mut self, frequency_source_id: usize) -> usize {
        let id = self.current_id;
        self.nodes
            .push(Rc::new(RefCell::new(SineOscillatorNode::new(
                id,
                frequency_source_id,
            ))));
        self.current_id += 1;
        id
    }

    pub fn insert_square_oscillator_node(&mut self, frequency_source_id: usize) -> usize {
        let id = self.current_id;
        self.nodes
            .push(Rc::new(RefCell::new(SquareOscillatorNode::new(
                id,
                frequency_source_id,
            ))));
        self.current_id += 1;
        id
    }

    pub fn insert_spline_float_node(
        &mut self,
        frequency_source_id: usize,
        points: Vec<Point>,
    ) -> usize {
        let id = self.current_id;
        self.nodes.push(Rc::new(RefCell::new(SplineFloatNode::new(
            id,
            frequency_source_id,
            points,
        ))));
        self.current_id += 1;
        id
    }

    pub fn insert_sum_node(&mut self, augend_source_id: usize, addend_source_id: usize) -> usize {
        let id = self.current_id;
        self.nodes.push(Rc::new(RefCell::new(SumNode::new(
            id,
            augend_source_id,
            addend_source_id,
        ))));
        self.current_id += 1;
        id
    }

    pub fn nodes(&self) -> Vec<Rc<RefCell<dyn Source>>> {
        self.nodes.clone()
    }
}
