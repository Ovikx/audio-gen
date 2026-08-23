use std::{fs::File, io::BufReader};

use hound::WavReader;

use crate::source::Source;

pub struct MediaNode {
    id: usize,
    dependency_ids: Vec<usize>,
    samples: Vec<f32>,
    sample_idx: usize,
}

impl MediaNode {
    pub fn new(id: usize, mut wav_reader: WavReader<BufReader<File>>) -> Self {
        let num_channels = wav_reader.spec().channels;
        dbg!(num_channels);
        let samples = wav_reader
            .samples::<f32>()
            .enumerate()
            .filter_map(|(idx, sample)| {
                if idx % num_channels as usize == 0 {
                    sample.ok()
                } else {
                    None
                }
            })
            .collect::<Vec<f32>>();
        MediaNode {
            id,
            dependency_ids: vec![],
            samples,
            sample_idx: 0,
        }
    }

    fn poll(&mut self) -> Option<f32> {
        let sample = if self.sample_idx < self.samples.len() {
            Some(self.samples[self.sample_idx])
        } else {
            None
        };
        self.sample_idx += 1;
        sample
    }
}

impl Source for MediaNode {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        _audio_context: &crate::context::AudioContext,
        _id_to_output: &crate::source::NodeOutput,
        output: &mut [Option<f32>],
    ) {
        for idx in 0..num_samples {
            output[idx] = self.poll();
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
