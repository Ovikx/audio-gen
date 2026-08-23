use nalgebra::{DMatrix, DVector};

pub type Point = (f32, f32);

pub fn spline_coefficients(sorted_points: &Vec<Point>) -> Vec<f32> {
    let num_points = sorted_points.len();
    let mut slopes = vec![0.0f32; num_points - 1];
    let mut derivatives = vec![0.0f32; num_points];
    let mut coefficients: Vec<f32> = vec![];

    for i in 0..num_points - 1 {
        slopes[i] = (sorted_points[i + 1].1 - sorted_points[i].1)
            / (sorted_points[i + 1].0 - sorted_points[i].0);
    }

    // Edge points don't need harmonic mean for derivative
    derivatives[0] = slopes[0];
    let last_idx = derivatives.len() - 1;
    derivatives[last_idx] = slopes[last_idx - 1];

    for i in 1..num_points - 1 {
        if (sorted_points[i].1 <= sorted_points[i - 1].1
            && sorted_points[i].1 <= sorted_points[i + 1].1)
            || (sorted_points[i].1 >= sorted_points[i - 1].1
                && sorted_points[i].1 >= sorted_points[i + 1].1)
        {
            derivatives[i] = 0.;
        } else {
            derivatives[i] = (2. * slopes[i] * slopes[i - 1]) / (slopes[i] + slopes[i - 1]);
        }
    }

    for i in 0..num_points - 1 {
        #[rustfmt::skip] // Preserve matrix formatting
        let lhs_matrix = DMatrix::from_row_slice(4, 4, &[
            sorted_points[i].0.powi(3), sorted_points[i].0.powi(2), sorted_points[i].0, 1.,
            sorted_points[i+1].0.powi(3), sorted_points[i+1].0.powi(2), sorted_points[i+1].0, 1.,
            3.*sorted_points[i].0.powi(2), 2.*sorted_points[i].0, 1., 0.,
            3.*sorted_points[i+1].0.powi(2), 2.*sorted_points[i+1].0, 1., 0.,
        ]);

        let rhs_vector = DVector::from_row_slice(&[
            sorted_points[i].1,
            sorted_points[i + 1].1,
            derivatives[i],
            derivatives[i + 1],
        ]);

        let lu = lhs_matrix.lu();
        let partial_coefficients = lu.solve(&rhs_vector).unwrap();
        for coefficient in partial_coefficients.iter() {
            coefficients.push(*coefficient);
        }
    }

    coefficients
}

#[cfg(test)]
mod tests {
    use crate::math::spline_polynomial::spline_coefficients;
    use test_utils::threshold_eq_float32;

    #[test]
    fn test_spline_coefficients() {
        let points = vec![(3., 3.), (6., 6.), (9., 4.), (12., 4.)];

        // Expected coefficients were computed using a Python reference function
        let expected_coefficients = [
            -0.11111111111111108f32,
            1.333333333333333,
            -4.,
            6.,
            0.14814814814814783,
            -3.3333333333333273,
            24.,
            -50.,
            1.370645709413773e-18,
            -4.5231308410654523e-17,
            4.934324553889585e-16,
            4.,
        ]
        .to_vec();

        let actual_coefficients = spline_coefficients(&points);

        assert_eq!(
            expected_coefficients.len(),
            actual_coefficients.len(),
            "expected coefficient array length of {}, got {}",
            expected_coefficients.len(),
            actual_coefficients.len()
        );

        for i in 0..expected_coefficients.len() {
            assert!(
                threshold_eq_float32(actual_coefficients[i], expected_coefficients[i]),
                "expected coefficient {}, got {}",
                expected_coefficients[i],
                actual_coefficients[i]
            )
        }
    }
}
