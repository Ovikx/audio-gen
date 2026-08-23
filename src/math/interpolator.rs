use crate::{
    context::AudioContext,
    math::spline_polynomial::{Point, spline_coefficients},
};

pub struct Interpolator {
    periodic: bool,
    x_values: Vec<f32>,
    coefficients: Vec<f32>,
    current_time: f32,
    current_x_value_idx: usize, // We keep track of the most recently used x-value index for interpolation
}

impl Interpolator {
    pub fn new(points: Vec<Point>, periodic: bool) -> Self {
        let mut sorted_points = points.clone();
        sorted_points.sort_by(|&p1, &p2| p1.0.total_cmp(&p2.0));
        let coefficients = spline_coefficients(&sorted_points);

        Interpolator {
            periodic,
            x_values: sorted_points.iter().map(|&point| point.0).collect(),
            coefficients: coefficients,
            current_time: 0.,
            current_x_value_idx: 0,
        }
    }

    pub fn next(&mut self, duration: f32, audio_context: &AudioContext) -> f32 {
        let frequency = 1.0 / duration;
        let mut used_x_value_idx = self.current_x_value_idx;
        let sample: f32;

        if used_x_value_idx > 0 && self.current_time <= self.x_values[used_x_value_idx] {
            // A cached index of 0 only appears as the default, so it should not be used
            sample =
                specific_interpolate(self.current_time, &self.coefficients, used_x_value_idx - 1);
        } else {
            (sample, used_x_value_idx) =
                general_interpolate(self.current_time, &self.x_values, &self.coefficients);
            self.current_x_value_idx = used_x_value_idx;
        }

        self.current_time += frequency / audio_context.sample_rate;

        // Incrementing the current time could have invalidated the cached x-value index, so we might need to update it
        if self.current_time > self.x_values[used_x_value_idx] {
            self.current_x_value_idx = if self.current_x_value_idx == self.x_values.len() - 1 {
                1
            } else {
                self.current_x_value_idx + 1
            };
        }

        if self.periodic {
            self.current_time = self.current_time.fract();
        } else if self.current_time > 1.0 {
            self.current_time = 1.0;
            return 0.0;
        }

        sample
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

#[cfg(test)]
mod tests {
    use test_utils::threshold_eq_float32;

    use super::*;

    #[test]
    fn test_periodic_spline_sequence() {
        let mut interpolator = Interpolator::new(vec![(0.0, 0.0), (1.0, 1.0)], true);

        let num_sets = 100;
        let sample_rate = 4.;

        let audio_context = AudioContext { sample_rate };
        let samples: Vec<f32> = (0..(4 * num_sets + 1))
            .map(|_| interpolator.next(1., &audio_context))
            .collect();

        let expected_samples: Vec<f32> = vec![0.25, 0.5, 0.75, 0.0];
        assert!(threshold_eq_float32(samples[0], 0.));
        for i in 1..num_sets * 4 {
            assert!(threshold_eq_float32(
                samples[i as usize],
                expected_samples[(((i - 1) as u32) % 4) as usize]
            ));
        }
    }

    #[test]
    fn test_aperiodic_spline_sequence() {
        let mut interpolator = Interpolator::new(vec![(0.0, 0.0), (1.0, 1.0)], false);

        let duration = 10.0001; // Ensure the interpolated values don't become zero at the last sample in the duration window due to floating point error
        let sample_rate = 1.0;
        let audio_context = AudioContext::new(sample_rate);
        let num_samples = 20;
        let samples: Vec<f32> = (0..num_samples)
            .map(|_| interpolator.next(duration, &audio_context))
            .collect();

        for i in 0..num_samples {
            if i < 10 {
                assert!(
                    threshold_eq_float32(samples[i], (i as f32) / 10.),
                    "samples[{}] = {}, expected {}",
                    i,
                    samples[i],
                    (i as f32) / 10.
                );
            } else {
                assert!(
                    threshold_eq_float32(samples[i], 0.),
                    "samples[{}] = {}, expected {}",
                    i,
                    samples[i],
                    0.
                );
            }
        }
    }
}
