use crate::{easing::Easing, rectdata::RectData};
use web_time::Duration;

const MAX_RATE_60HZ: u64 = 60;

#[derive(Clone, PartialEq, Debug)]
pub struct AnimationTransition {
    pub from: RectData,
    pub to: RectData,
    pub easing: Easing,
    pub start_delay: Option<web_time::Duration>,
    duration: web_time::Duration,
    min_frame_duration: web_time::Duration,
    linear_progress: f32,
}

impl AnimationTransition {
    pub fn new(from: RectData, to: RectData, duration: web_time::Duration, easing: Easing) -> Self {
        let min_frame_duration = Self::get_frame_duration_from_refresh_rate(MAX_RATE_60HZ);
        Self {
            from,
            to,
            easing,
            start_delay: None,
            duration,
            min_frame_duration,
            linear_progress: 0f32,
        }
    }

    pub fn with_delay(mut self, duration: web_time::Duration) -> Self {
        self.start_delay = Some(duration);
        self
    }

    #[allow(unused)]
    pub fn with_refresh_rate(mut self, max_refresh_rate: u64) -> Self {
        self.min_frame_duration = Self::get_frame_duration_from_refresh_rate(max_refresh_rate);
        self
    }

    fn get_frame_duration_from_refresh_rate(max_refresh_rate: u64) -> Duration {
        Duration::from_millis(1000 / max_refresh_rate)
    }

    pub async fn step(&mut self, total_elapsed: web_time::Duration) -> RectData {
        let frame_start = web_time::SystemTime::now();

        self.linear_progress =
            (total_elapsed.as_secs_f64() / self.duration.as_secs_f64()).clamp(0., 1.) as f32;
        let interpolated_progress = self.easing.ease(self.linear_progress);

        let current_rect = self.from.interpolate_to(interpolated_progress, &self.to);

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

    pub fn move_x_linear() -> Self {
        let from = RectData::new(0f64, 0f64, 200f64, 200f64);
        let to = RectData::new(400f64, 200f64, 200f64, 100f64);
        let duration = web_time::Duration::from_millis(1000);
        Self::new(from, to, duration, Easing::ElasticOut)
    }
}
