use std::{collections::HashMap, path::PathBuf};

use hound::WavReader;

use crate::{
    graph::node_signature::{NodeSignature, NodeType::*},
    input_buffer::SharedExternalInputBuffer,
    math::spline_polynomial::Point,
    node::{
        ADSRNode, AbsoluteValue, ExternalFloatNode, FilterType, FloatSource, FreeverbNode,
        MediaNode, MultiplyNode, NoiseNode, SVFNode, SawOscillatorNode, SequenceNode,
        SineOscillatorNode, SourceInterval, SplineFloatNode, SquareOscillatorNode, SumNode,
    },
    source::Source,
};

pub struct Graph {
    current_id: usize,
    nodes: Vec<Box<dyn Source>>,
    dedupe: bool,
    signature_to_id: HashMap<NodeSignature, usize>,
}

impl Graph {
    pub fn new(dedupe: bool) -> Self {
        Graph {
            current_id: 0,
            nodes: vec![],
            dedupe,
            signature_to_id: HashMap::new(),
        }
    }

    pub fn float_node(&mut self, value: f32) -> usize {
        let signature = NodeSignature::new_with_data(Float, vec![], value.to_string());
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            self.nodes.push(Box::new(FloatSource::new(id, value)));
            self.current_id += 1;
        }
        id
    }

    pub fn external_float_node(
        &mut self,
        input_buffer: SharedExternalInputBuffer,
        input_buffer_index: usize,
    ) -> usize {
        let id = self.current_id;
        self.nodes.push(Box::new(ExternalFloatNode::new(
            id,
            input_buffer,
            input_buffer_index,
        )));
        self.current_id += 1;
        id
    }

    pub fn multiply_node(
        &mut self,
        multiplicand_source_id: usize,
        multiplier_source_id: usize,
    ) -> usize {
        let signature =
            NodeSignature::new(Multiply, vec![multiplicand_source_id, multiplier_source_id]);
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            self.nodes.push(Box::new(MultiplyNode::new(
                id,
                multiplicand_source_id,
                multiplier_source_id,
            )));
            self.current_id += 1;
        }
        id
    }

    pub fn saw_oscillator_node(&mut self, frequency_source_id: usize) -> usize {
        let signature = NodeSignature::new(Saw, vec![frequency_source_id]);
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            self.nodes
                .push(Box::new(SawOscillatorNode::new(id, frequency_source_id)));
            self.current_id += 1;
        }
        id
    }

    pub fn sine_oscillator_node(&mut self, frequency_source_id: usize) -> usize {
        let signature = NodeSignature::new(Sine, vec![frequency_source_id]);
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            self.nodes
                .push(Box::new(SineOscillatorNode::new(id, frequency_source_id)));
            self.current_id += 1;
        }
        id
    }

    pub fn square_oscillator_node(&mut self, frequency_source_id: usize) -> usize {
        let signature = NodeSignature::new(Square, vec![frequency_source_id]);
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            self.nodes
                .push(Box::new(SquareOscillatorNode::new(id, frequency_source_id)));
            self.current_id += 1;
        }
        id
    }

    pub fn spline_float_node(&mut self, frequency_source_id: usize, points: Vec<Point>) -> usize {
        let signature = NodeSignature::new_with_data(
            Spline,
            vec![frequency_source_id],
            points.iter().map(|point| format!("{:?}", point)).collect(),
        );
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            self.nodes.push(Box::new(SplineFloatNode::new(
                id,
                frequency_source_id,
                points,
            )));
            self.current_id += 1;
        }
        id
    }

    pub fn sum_node(&mut self, augend_source_id: usize, addend_source_id: usize) -> usize {
        let signature = NodeSignature::new(Sum, vec![augend_source_id, addend_source_id]);
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            self.nodes.push(Box::new(SumNode::new(
                id,
                augend_source_id,
                addend_source_id,
            )));
            self.current_id += 1;
        }
        id
    }

    pub fn noise_node(&mut self, seed: u64) -> usize {
        let signature = NodeSignature::new_with_data(Noise, vec![], seed.to_string());
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            self.nodes.push(Box::new(NoiseNode::new(id, seed)));
            self.current_id += 1;
        }
        id
    }

    pub fn absolute_value_node(&mut self, source_id: usize) -> usize {
        let signature = NodeSignature::new(AbsoluteValue, vec![source_id]);
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            self.nodes.push(Box::new(AbsoluteValue::new(id, source_id)));
            self.current_id += 1;
        }
        id
    }

    pub fn svf_node(
        &mut self,
        filter_type: FilterType,
        sample_source_id: usize,
        frequency_cutoff_source_id: usize,
        resonance_source_id: usize,
    ) -> usize {
        let signature = NodeSignature::new_with_data(
            SVF,
            vec![
                sample_source_id,
                frequency_cutoff_source_id,
                resonance_source_id,
            ],
            format!("{:?}", filter_type),
        );
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            self.nodes.push(Box::new(SVFNode::new(
                id,
                filter_type,
                sample_source_id,
                frequency_cutoff_source_id,
                resonance_source_id,
            )));
            self.current_id += 1;
        }
        id
    }

    pub fn sequence_node(&mut self, source_intervals: Vec<SourceInterval>) -> usize {
        let signature = NodeSignature::new_with_data(
            Sequence,
            vec![],
            source_intervals
                .iter()
                .map(|interval| format!("{:?}", interval))
                .collect(),
        );
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            self.nodes
                .push(Box::new(SequenceNode::new(id, source_intervals)));
            self.current_id += 1;
        }
        id
    }

    pub fn freeverb_node(
        &mut self,
        sample_source_id: usize,
        room_size_source_id: usize,
        damping_source_id: usize,
        wet_source_id: usize,
        dry_source_id: usize,
    ) -> usize {
        let signature = NodeSignature::new(
            Freeverb,
            vec![
                sample_source_id,
                room_size_source_id,
                damping_source_id,
                wet_source_id,
                dry_source_id,
            ],
        );
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            self.nodes.push(Box::new(FreeverbNode::new(
                id,
                sample_source_id,
                room_size_source_id,
                damping_source_id,
                wet_source_id,
                dry_source_id,
            )));
            self.current_id += 1;
        }
        id
    }

    pub fn adsr_node(
        &mut self,
        sample_source_id: usize,
        duration: f32,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    ) -> usize {
        let signature = NodeSignature::new(ADSR, vec![sample_source_id]);
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            self.nodes.push(Box::new(ADSRNode::new(
                id,
                sample_source_id,
                duration,
                attack,
                decay,
                sustain,
                release,
            )));
            self.current_id += 1;
        }
        id
    }

    pub fn media_node(&mut self, wav_file_path: PathBuf) -> usize {
        let signature = NodeSignature::new_with_data(
            Media,
            vec![],
            wav_file_path
                .to_str()
                .expect("expected wav file to resolve to str")
                .to_string(),
        );
        let (id, signature_exists) = self.fetch_signature_id(signature);
        if !signature_exists {
            let reader =
                WavReader::open(wav_file_path).expect("media node received invalid file path: {}");
            self.nodes.push(Box::new(MediaNode::new(id, reader)));
            self.current_id += 1;
        }
        id
    }

    pub fn nodes(self) -> Vec<Box<dyn Source>> {
        self.nodes
    }

    fn fetch_signature_id(&mut self, signature: NodeSignature) -> (usize, bool) {
        if !self.dedupe {
            return (self.current_id, false);
        }

        if let Some(signature_id) = self.signature_to_id.get(&signature) {
            return (*signature_id, true);
        }
        self.signature_to_id.insert(signature, self.current_id);
        (self.current_id, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedupe_chain() {
        let num_chains = 10;
        let mut graph = Graph::new(true);
        for _ in 0..num_chains {
            let multiplicand_id = graph.float_node(1.0);
            let mulitplier_id = graph.float_node(2.0);
            graph.multiply_node(multiplicand_id, mulitplier_id);
        }

        assert_eq!(graph.nodes().len(), 3);
    }
}
