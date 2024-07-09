use crate::easing::Easing;
use dioxus::html::geometry::euclid::Rect;
use web_time::Duration;

use super::AnimationBuilder;

pub const MAX_RATE_60HZ: u64 = 60;
pub const MAX_RATE_90HZ: u64 = 90;
pub const MAX_RATE_120HZ: u64 = 120;

#[derive(Clone, PartialEq, Debug)]
pub struct AnimationTransition {
    pub from: Rect<f64, f64>,
    pub to: Rect<f64, f64>,
    pub easing: Easing,
    duration: web_time::Duration,
    min_frame_duration: web_time::Duration,
    linear_progress: f32,
}

impl AnimationTransition {
    pub fn new(builder: AnimationBuilder, from: Rect<f64, f64>, to: Rect<f64, f64>) -> Self {
        let min_frame_duration = Self::get_frame_duration_from_refresh_rate(builder.fps_cap);
        Self {
            from,
            to,
            easing: builder.easing,
            duration: builder.duration,
            min_frame_duration,
            linear_progress: 0f32,
        }
    }

    fn get_frame_duration_from_refresh_rate(max_refresh_rate: u64) -> Duration {
        Duration::from_millis(1000 / max_refresh_rate)
    }

    pub async fn step(&mut self, total_elapsed: web_time::Duration) -> Rect<f64, f64> {
        let frame_start = web_time::SystemTime::now();

        self.linear_progress =
            (total_elapsed.as_secs_f64() / self.duration.as_secs_f64()).clamp(0., 1.) as f32;
        let interpolated_progress = self.easing.ease(self.linear_progress) as f64;

        let current_rect = self.from.lerp(self.to, interpolated_progress);

        let frame_duration = frame_start
            .elapsed()
            .expect("couldn't get elapsed time during frame");

        if frame_duration < self.min_frame_duration {
            let delay_duration = self.min_frame_duration.as_millis() - frame_duration.as_millis();
            let delay =
                gloo_timers::future::sleep(web_time::Duration::from_millis(delay_duration as u64));
            delay.await;
        }

        current_rect
    }

    pub fn is_finished(&self) -> bool {
        self.linear_progress >= 1.0
    }
}
