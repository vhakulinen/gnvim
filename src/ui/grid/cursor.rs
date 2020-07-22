use gdk;

use crate::ui::color::Color;

#[derive(Default)]
struct Animation {
    start: (f64, f64),
    end: (f64, f64),
    start_time: i64,
    end_time: i64,
}

#[derive(Default)]
pub struct Cursor {
    /// Position, (row, col).
    pub pos: (f64, f64),
    got_first_goto: bool,
    animation: Animation,

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
    pub fn goto(&mut self, row: f64, col: f64, clock: &gdk::FrameClock) {
        if !self.got_first_goto {
            self.pos = (row, col);
            self.got_first_goto = true;
        }

        let now = clock.get_frame_time();
        let duration = 200;
        self.animation = Animation {
            start: self.pos,
            end: (row, col),
            start_time: now,
            end_time: now + 1000 * duration,
        };
    }

    pub fn tick(&mut self, clock: &gdk::FrameClock) {
        self.blink();
        self.animate_position(clock);
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

    fn animate_position(&mut self, clock: &gdk::FrameClock) {
        let now = clock.get_frame_time();
        if now < self.animation.end_time && self.pos != self.animation.end {
            let mut t = (now - self.animation.start_time) as f64
                / (self.animation.end_time - self.animation.start_time) as f64;
            t = ease_out_cubic(t);
            self.pos.0 = self.animation.start.0
                + t * (self.animation.end.0 - self.animation.start.0);
            self.pos.1 = self.animation.start.1
                + t * (self.animation.end.1 - self.animation.start.1);
        } else if self.pos != self.animation.end {
            self.pos = self.animation.end;
        }
    }
}

/// From clutter-easing.c, based on Robert Penner's
/// infamous easing equations, MIT license.
fn ease_out_cubic(t: f64) -> f64 {
    let p = t - 1f64;
    return p * p * p + 1f64;
}
