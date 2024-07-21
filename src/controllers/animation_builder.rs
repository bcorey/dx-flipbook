use dioxus::html::geometry::euclid::Rect;

use crate::easing::Easing;

use super::{MAX_RATE_120HZ, MAX_RATE_60HZ, MAX_RATE_90HZ};

#[derive(Clone, PartialEq, Debug)]
pub struct AnimationBuilder {
    pub from: Option<Rect<f64, f64>>,
    pub to: Option<Rect<f64, f64>>,
    pub duration: web_time::Duration,
    pub easing: Easing,
    pub fps_cap: u64,
}

impl Default for AnimationBuilder {
    fn default() -> Self {
        Self {
            from: None,
            to: None,
            duration: web_time::Duration::from_millis(1000),
            easing: Easing::SineInOut,
            fps_cap: MAX_RATE_60HZ,
        }
    }
}

impl AnimationBuilder {
    pub fn new_delay(delay: web_time::Duration) -> Self {
        Self::default().with_duration(delay)
    }

    pub fn animate_from(mut self, from: Rect<f64, f64>) -> Self {
        self.from = Some(from);
        self
    }

    pub fn animate_to(mut self, to: Rect<f64, f64>) -> Self {
        self.to = Some(to);
        self
    }

    pub fn with_duration(mut self, duration: web_time::Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn at_max_90hz(mut self) -> Self {
        self.fps_cap = MAX_RATE_90HZ;
        self
    }

    pub fn at_max_120hz(mut self) -> Self {
        self.fps_cap = MAX_RATE_120HZ;
        self
    }
}
