#[derive(Debug, Hash, PartialEq, Eq)]
pub struct NodeSignature {
    node_type: NodeType,
    dependency_ids: Vec<usize>,
    ancillary_data: Option<String>,
}

impl NodeSignature {
    pub fn new(node_type: NodeType, dependency_ids: Vec<usize>) -> Self {
        NodeSignature {
            node_type,
            dependency_ids,
            ancillary_data: None,
        }
    }

    pub fn new_with_data(
        node_type: NodeType,
        dependency_ids: Vec<usize>,
        ancillary_data: String,
    ) -> Self {
        NodeSignature {
            node_type,
            dependency_ids,
            ancillary_data: Some(ancillary_data),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum NodeType {
    AbsoluteValue,
    Float,
    Freeverb,
    Multiply,
    Noise,
    Saw,
    Sequence,
    Sine,
    Spline,
    Square,
    Sum,
    SVF,
    ADSR,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_signature_equivalence() {
        let signature1 = NodeSignature::new(NodeType::Sine, vec![0]);
        let signature2 = NodeSignature::new(NodeType::Sine, vec![0]);
        assert_eq!(signature1, signature2);
    }

    #[test]
    fn test_node_signature_ancillary_difference() {
        let signature1 = NodeSignature::new(NodeType::Sine, vec![0]);
        let signature2 = NodeSignature::new_with_data(
            NodeType::Sine,
            vec![0],
            String::from("some differentiating data"),
        );
        assert_ne!(signature1, signature2);
    }

    #[test]
    fn test_node_signature_dependency_difference() {
        let signature1 = NodeSignature::new(NodeType::Sine, vec![0]);
        let signature2 = NodeSignature::new(NodeType::Sine, vec![1]);
        assert_ne!(signature1, signature2);
    }

    #[test]
    fn test_node_signature_node_type_difference() {
        let signature1 = NodeSignature::new(NodeType::Sine, vec![0]);
        let signature2 = NodeSignature::new(NodeType::Saw, vec![0]);
        assert_ne!(signature1, signature2);
    }
}
