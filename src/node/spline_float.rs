use crate::{
    context::audio_context::AudioContext,
    math::spline_polynomial::{Point, spline_coefficients},
    source::{LogicalTimestamp, SharedCachedFloatSource, Source},
};

pub struct SplineFloatNode {
    frequency_source: SharedCachedFloatSource,
    x_values: Vec<f32>,
    coefficients: Vec<f32>,
    current_time: f32,
}

impl SplineFloatNode {
    pub fn new(frequency_source: SharedCachedFloatSource, points: Vec<Point>) -> Self {
        let mut sorted_points = points.clone();
        sorted_points.sort_by(|&p1, &p2| p1.0.total_cmp(&p2.0));
        let coefficients = spline_coefficients(&sorted_points);

        SplineFloatNode {
            frequency_source,
            x_values: sorted_points.iter().map(|&point| point.0).collect(),
            coefficients: coefficients.to_owned(),
            current_time: 0.,
        }
    }
}

impl Source<f32> for SplineFloatNode {
    fn poll(&mut self, audio_context: &AudioContext, timestamp: LogicalTimestamp) -> Option<f32> {
        self.frequency_source
            .borrow_mut()
            .poll(audio_context, timestamp)
            .map(|f| {
                let current_time = self.current_time;
                let sample = interpolate(current_time, &self.x_values, &self.coefficients);
                self.current_time += f / audio_context.sample_rate;
                self.current_time -= 1.0 * self.current_time.floor();
                sample
            })
    }
}

fn interpolate(value: f32, x_values: &Vec<f32>, coefficients: &Vec<f32>) -> f32 {
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
            return a * value.powi(3) + b * value.powi(2) + c * value + d;
        }
    }
    return 0.;
}
