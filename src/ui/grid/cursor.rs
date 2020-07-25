use crate::ui::color::Color;

#[derive(Default)]
pub struct Animation {
    start: (f64, f64),
    end: (f64, f64),
    start_time: i64,
    end_time: i64,
}

#[derive(Default)]
pub struct Cursor {
    /// Position, (row, col).
    pub pos: Option<(f64, f64)>,
    /// Flag for disabling the movement animation.
    pub disable_animation: bool,
    pub animation: Option<Animation>,

    /// Alpha color. Used to make the cursor blink.
    pub alpha: f64,
    /// The duration of the blink.
    pub blink_on: u64,
    /// Width of the cursor.
    pub cell_percentage: f64,
    /// Color of the cursor.
    pub color: Color,
}

impl Cursor {
    pub fn goto(&mut self, row: f64, col: f64, frame_time: i64) {
        // When we get our first cursor_goto, set the position directly.
        if self.pos.is_none() {
            self.pos = Some((row, col));
        }

        // If cursor animation is disabled, set the position directly. Otherwise, set the animation
        // so that we can animate cursor position change.
        if self.disable_animation {
            self.pos = Some((row, col));
        } else {
            let duration = 100;
            self.animation = Some(Animation {
                start: self.pos.unwrap(),
                end: (row, col),
                start_time: frame_time,
                end_time: frame_time + 1000 * duration,
            });
        }
    }

    pub fn tick(&mut self, frame_time: i64) {
        self.blink();
        self.animate_position(frame_time);
    }

    fn blink(&mut self) {
        // If we dont need to blink, return.
        if self.blink_on == 0 {
            return;
        }

        // Assuming a 60hz framerate
        self.alpha += 100.0 / (6.0 * self.blink_on as f64);

        if self.alpha > 2.0 {
            self.alpha = 0.0;
        }
    }

    fn animate_position(&mut self, frame_time: i64) {
        if let Some(Animation {
            start,
            end,
            start_time,
            end_time,
        }) = self.animation
        {
            let mut pos = self.pos.unwrap_or((0.0, 0.0));

            if frame_time < end_time && pos != end {
                let mut t = (frame_time - start_time) as f64
                    / (end_time - start_time) as f64;
                t = ease_out_cubic(t);
                pos.0 = start.0 + t * (end.0 - start.0);
                pos.1 = start.1 + t * (end.1 - start.1);

                self.pos = Some(pos);
            } else {
                self.pos = Some(end);
                self.animation = None;
            }
        }
    }

    /// Gets the position of the cursor.
    pub fn get_position(&self) -> Option<(f64, f64)> {
        if let Some(ref a) = self.animation {
            // The end position of our animation is the "real" position where
            // the cursor is.
            Some(a.end)
        } else {
            self.pos
        }
    }
}

/// From clutter-easing.c, based on Robert Penner's
/// infamous easing equations, MIT license.
fn ease_out_cubic(t: f64) -> f64 {
    let p = t - 1f64;
    p * p * p + 1f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_blink100() {
        let mut cursor = Cursor::default();
        cursor.blink_on = 100;
        cursor.alpha = 1.0;

        cursor.blink();
        assert_eq!(cursor.alpha, 1.1666666666666667);
    }

    #[test]
    fn test_cursor_blink0() {
        let mut cursor = Cursor::default();
        cursor.blink_on = 0;
        cursor.alpha = 1.0;

        cursor.blink();
        assert_eq!(cursor.alpha, 1.0);
    }

    #[test]
    fn test_first_position() {
        let mut cursor = Cursor::default();

        // When we first set the position, it should be set immediately.
        cursor.goto(15.0, 15.0, 1);
        assert_eq!(cursor.pos, Some((15.0, 15.0)));

        // When we've set the position once already, the subsequent goto positions should be set
        // with some delay by the animation.
        cursor.goto(10.0, 10.0, 1);
        assert_eq!(cursor.pos, Some((15.0, 15.0)));
    }

    #[test]
    fn test_animate_position() {
        let mut cursor = Cursor::default();

        // When we first set the position, it should be set immediately.
        cursor.goto(15.0, 15.0, 1);
        assert_eq!(cursor.pos, Some((15.0, 15.0)));

        cursor.goto(10.0, 10.0, 1);
        cursor.tick(25000);
        assert_eq!(cursor.pos, Some((12.109459376125006, 12.109459376125006)));
    }

    #[test]
    fn test_animate_position_animation_disabled() {
        let mut cursor = Cursor::default();
        cursor.disable_animation = true;

        // When we first set the position, it should be set immediately.
        cursor.goto(15.0, 15.0, 1);
        assert_eq!(cursor.pos, Some((15.0, 15.0)));

        // Position animation is disabled, goto should change the position directly and tick
        // shouldn't affect the position value at all.
        cursor.goto(10.0, 10.0, 1);
        assert_eq!(cursor.pos, Some((10.0, 10.0)));
        cursor.tick(25000);
        assert_eq!(cursor.pos, Some((10.0, 10.0)));
    }

    #[test]
    fn test_get_position() {
        let mut cursor = Cursor::default();

        assert_eq!(cursor.get_position(), None);
        cursor.pos = Some((10.0, 10.0));
        assert_eq!(cursor.get_position(), Some((10.0, 10.0)));
        cursor.animation = Some(Animation {
            end: (15.0, 15.0),
            ..Animation::default()
        });
        assert_eq!(cursor.get_position(), Some((15.0, 15.0)));
    }
}
