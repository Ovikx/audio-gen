#[derive(Clone, Copy)]
pub struct AudioContext {
    pub sample_rate: f32,
}

impl AudioContext {
    pub fn new(sample_rate: f32) -> Self {
        AudioContext { sample_rate }
    }
}
