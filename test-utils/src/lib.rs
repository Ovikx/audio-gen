pub fn threshold_eq_float32(value: f32, target: f32) -> bool {
    let margin = 0.0001;
    target - margin <= value && target + margin >= value
}
