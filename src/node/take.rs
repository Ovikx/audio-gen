// use crate::{
//     context::audio_context::AudioContext,
//     node::source::{SharedFloatSource, Source},
// };

// pub struct TakeSource {
//     sample_source: SharedFloatSource,
//     num_samples: u32,
// }

// impl TakeSource {
//     pub fn new(sample_source: SharedFloatSource, num_samples: u32) -> Self {
//         TakeSource {
//             sample_source,
//             num_samples,
//         }
//     }
// }

// impl Source<f32> for TakeSource {
//     fn poll(&mut self, _audio_context: &AudioContext) -> Option<f32> {}
// }
