#[derive(Default)]
pub struct Animation<T> {
    pub start: T,
    pub end: T,
    pub start_time: i64,
    pub end_time: i64,
}

impl<T> Animation<T> {
    pub fn tick(&self, frame_time: i64) -> Option<f64> {
        if frame_time < self.end_time {
            let t = (frame_time - self.start_time) as f64
                / (self.end_time - self.start_time) as f64;
            Some(ease_out_cubic(t))
        } else {
            None
        }
    }
}

/// From clutter-easing.c, based on Robert Penner's
/// infamous easing equations, MIT license.
fn ease_out_cubic(t: f64) -> f64 {
    let p = t - 1f64;
    p * p * p + 1f64
}
