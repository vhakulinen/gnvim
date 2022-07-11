pub fn ease_out_cubic(t: f64) -> f64 {
    1.0 + (t - 1.0).powi(3)
}
