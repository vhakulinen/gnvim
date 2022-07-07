use crate::warn;

#[derive(Default, Debug)]
pub struct Blink {
    wait: f64,
    on: f64,
    off: f64,
    transition: f64,

    stage: Stage,
    stage_end: f64,
    stage_start: f64,
    pub alpha: f64,
}

fn ease_out_cubic(t: f64) -> f64 {
    1.0 + (t - 1.0).powi(3)
}

#[derive(Default, Debug)]
enum Stage {
    #[default]
    Wait,
    Off,
    GoingOn,
    On,
    GoingOff,
}

impl Stage {
    fn next(&self) -> Self {
        match self {
            Self::Wait | Self::On => Self::GoingOff,
            Self::GoingOff => Self::Off,
            Self::Off => Self::GoingOn,
            Self::GoingOn => Self::On,
        }
    }
}

impl Blink {
    pub fn new(wait: f64, on: f64, off: f64, transition: f64, t: f64) -> Option<Self> {
        if wait == 0.0 && on == 0.0 && off == 0.0 {
            None
        } else {
            let mut blink = Self {
                wait,
                on,
                off,
                transition,
                alpha: 1.0,
                ..Default::default()
            };

            blink.set_stage(Stage::Wait, t);

            Some(blink)
        }
    }

    fn set_stage(&mut self, stage: Stage, frame_time: f64) {
        let duration = match stage {
            Stage::Wait => self.wait,
            Stage::GoingOff | Stage::GoingOn => self.transition,
            Stage::On => self.on,
            Stage::Off => self.off,
        };

        self.stage = stage;
        self.stage_start = frame_time;
        self.stage_end = frame_time + duration;
    }

    pub fn reset_to_wait(&mut self, t: f64) {
        self.set_stage(Stage::Wait, t);
        self.alpha = 1.0;
    }

    pub fn tick(&mut self, t: f64) {
        if t < self.stage_start {
            warn!("Clock going backwards");
            return;
        }

        if self.stage_end < t {
            self.set_stage(self.stage.next(), t);
            return;
        }

        match self.stage {
            Stage::GoingOff => {
                let t = (t - self.stage_start) / (self.stage_end - self.stage_start);
                let t = ease_out_cubic(t);
                let alpha = 1.0 - t;
                self.alpha = alpha.max(0.0);
            }
            Stage::GoingOn => {
                let t = (t - self.stage_start) / (self.stage_end - self.stage_start);
                let t = ease_out_cubic(t);
                let alpha = t;
                self.alpha = alpha.min(1.0);
            }
            Stage::Off => self.alpha = 0.0,
            Stage::On => self.alpha = 1.0,
            Stage::Wait => (),
        }
    }

    /// Get the cursor blink's wait.
    pub fn wait(&self) -> f64 {
        self.wait
    }

    /// Get the cursor blink's on.
    pub fn on(&self) -> f64 {
        self.on
    }

    /// Get the cursor blink's off.
    pub fn off(&self) -> f64 {
        self.off
    }

    /// Get the cursor blink's transition.
    pub fn transition(&self) -> f64 {
        self.transition
    }
}
