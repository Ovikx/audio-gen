use crate::{
    context::AudioContext,
    math::spline_polynomial::{Point, spline_coefficients},
    source::Source,
};

pub struct SplineFloatNode {
    id: usize,
    frequency_source_id: usize,
    dependency_ids: Vec<usize>,
    x_values: Vec<f32>,
    coefficients: Vec<f32>,
    current_time: f32,
    current_x_value_idx: usize, // We keep track of the most recently used x-value index for interpolation
}

impl SplineFloatNode {
    pub fn new(id: usize, frequency_source_id: usize, points: Vec<Point>) -> Self {
        let mut sorted_points = points.clone();
        sorted_points.sort_by(|&p1, &p2| p1.0.total_cmp(&p2.0));
        let coefficients = spline_coefficients(&sorted_points);

        SplineFloatNode {
            id,
            frequency_source_id,
            dependency_ids: vec![frequency_source_id],
            x_values: sorted_points.iter().map(|&point| point.0).collect(),
            coefficients: coefficients.to_owned(),
            current_time: 0.,
            current_x_value_idx: 0, // This works as a default since all values will be larger than the first x-value, leading to a cache invalidation
        }
    }
}

impl Source for SplineFloatNode {
    fn poll(
        &mut self,
        audio_context: &AudioContext,
        id_to_output: &crate::source::NodeOutput,
    ) -> Option<f32> {
        id_to_output[self.frequency_source_id].map(|f| {
            let mut used_x_value_idx = self.current_x_value_idx;
            let sample: f32;

            if used_x_value_idx > 0 && self.current_time <= self.x_values[used_x_value_idx] {
                // A cached index of 0 only appears as the default, so it should not be used
                sample = specific_interpolate(
                    self.current_time,
                    &self.coefficients,
                    used_x_value_idx - 1,
                );
            } else {
                (sample, used_x_value_idx) =
                    general_interpolate(self.current_time, &self.x_values, &self.coefficients);
            }

            self.current_time += f / audio_context.sample_rate;

            // Incrementing the current time could have invalidated the cached x-value index, so we might need to update it
            if self.current_time > self.x_values[used_x_value_idx] {
                self.current_x_value_idx = if self.current_x_value_idx == self.x_values.len() - 1 {
                    1
                } else {
                    self.current_x_value_idx + 1
                };
            }

            self.current_time -= 1.0 * self.current_time.floor();
            sample
        })
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}

fn general_interpolate(value: f32, x_values: &Vec<f32>, coefficients: &Vec<f32>) -> (f32, usize) {
    assert!(
        value >= x_values[0] && value <= x_values[x_values.len() - 1],
        "{} is not between {} and {}",
        value,
        x_values[0],
        x_values[x_values.len() - 1]
    );
    for i in 1..x_values.len() {
        if value <= x_values[i] {
            let a = coefficients[4 * (i - 1)];
            let b = coefficients[4 * (i - 1) + 1];
            let c = coefficients[4 * (i - 1) + 2];
            let d = coefficients[4 * (i - 1) + 3];
            return (a * value.powi(3) + b * value.powi(2) + c * value + d, i);
        }
    }
    (0., 0)
}

fn specific_interpolate(value: f32, coefficients: &Vec<f32>, coefficient_chunk_idx: usize) -> f32 {
    let a = coefficients[4 * coefficient_chunk_idx];
    let b = coefficients[4 * coefficient_chunk_idx + 1];
    let c = coefficients[4 * coefficient_chunk_idx + 2];
    let d = coefficients[4 * coefficient_chunk_idx + 3];
    return a * value.powi(3) + b * value.powi(2) + c * value + d;
}
