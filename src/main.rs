#![allow(non_snake_case)]

use dioxus::prelude::*;
use tracing::Level;
use web_time::Duration;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

#[component]
fn App() -> Element {
    let mut animatable = use_signal(|| AnimatableState::default());
    rsx! {
        Animatable {
            animation: animatable,
            div {
                style: "background-color: red; width: 100%; height: 100%;",
            }
        }
        button {
            onclick: move |_| animatable.write().to_400(),
            "animate"
        }
        button {
            onclick: move |_| animatable.write().toggle_play_state(),
            "play/pause"
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct RectData {
    size: RectSize,
    position: RectPos,
}

impl RectData {
    fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            size: RectSize::new(width, height),
            position: RectPos::new(x, y),
        }
    }

    fn to_css(&self) -> String {
        format!("{} {}", self.size.to_css(), self.position.to_css())
    }
}

#[derive(Clone, PartialEq, Debug)]
struct RectPos {
    x: f64,
    y: f64,
}

impl RectPos {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn to_css(&self) -> String {
        format!("top: {}px; left: {}px;", self.y, self.x)
    }
}

#[derive(Clone, PartialEq, Debug)]
struct RectSize {
    width: f64,
    height: f64,
}

impl RectSize {
    fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    fn to_css(&self) -> String {
        format!("width: {}px; height: {}px;", self.width, self.height)
    }
}

const MAX_RATE_60HZ: u64 = 60;
#[derive(Clone, PartialEq, Debug)]
struct AnimationTransition {
    from: RectData,
    to: RectData,
    duration: web_time::Duration,
    min_frame_duration: web_time::Duration,
    linear_progress: f32,
    start_time: web_time::SystemTime,
    state: AnimationPlayState,
}

impl AnimationTransition {
    fn new(from: RectData, to: RectData) -> Self {
        let min_frame_duration = Self::get_frame_duration_from_refresh_rate(MAX_RATE_60HZ);
        Self {
            from,
            to,
            duration: web_time::Duration::from_millis(500u64),
            min_frame_duration,
            linear_progress: 0f32,
            start_time: web_time::SystemTime::now(),
            state: AnimationPlayState::Play,
        }
    }

    #[allow(unused)]
    fn with_refresh_rate(mut self, max_refresh_rate: u64) -> Self {
        self.min_frame_duration = Self::get_frame_duration_from_refresh_rate(max_refresh_rate);
        self
    }

    fn get_frame_duration_from_refresh_rate(max_refresh_rate: u64) -> Duration {
        Duration::from_millis(1000 / max_refresh_rate)
    }

    async fn step(&mut self) -> RectData {
        let frame_start = web_time::SystemTime::now();
        let total_elapsed = self
            .start_time
            .elapsed()
            .expect("could not get elapsed time since animation start");

        if total_elapsed >= self.duration {
            return self.to.clone();
        }

        self.linear_progress = (total_elapsed.as_secs_f64() / self.duration.as_secs_f64()) as f32;
        let interpolated_progress = simple_easing::bounce_out(self.linear_progress) as f64;

        let mut current_rect = self.from.clone();
        let total_x_diff = self.to.position.x - self.from.position.x;
        current_rect.position.x += total_x_diff * interpolated_progress;

        let frame_duration = frame_start
            .elapsed()
            .expect("couldn't get elapsed time during frame");

        if frame_duration < self.min_frame_duration {
            let delay_duration = self.min_frame_duration.as_millis() - frame_duration.as_millis();
            let delay = gloo_timers::future::sleep(Duration::from_millis(delay_duration as u64));
            delay.await;
        }

        current_rect
    }

    fn move_x_linear() -> Self {
        let from = RectData::new(0f64, 0f64, 200f64, 200f64);
        let to = RectData::new(400f64, 0f64, 200f64, 200f64);
        Self::new(from, to)
    }
}

#[derive(Clone, PartialEq, Debug)]
struct AnimatableState {
    animation: Option<AnimationTransition>,
}

impl Default for AnimatableState {
    fn default() -> Self {
        Self { animation: None }
    }
}

impl AnimatableState {
    fn to_400(&mut self) {
        self.animation = Some(AnimationTransition::move_x_linear());
    }

    fn toggle_play_state(&mut self) {
        self.animation
            .as_mut()
            .map(|animation| animation.state = animation.state.toggle());
    }
}

#[derive(Clone, PartialEq, Debug)]
enum AnimationPlayState {
    Play,
    Pause,
}

impl AnimationPlayState {
    fn toggle(&self) -> Self {
        match self {
            Self::Play => Self::Pause,
            Self::Pause => Self::Play,
        }
    }
}

#[component]
fn Animatable(animation: Signal<AnimatableState>, children: Element) -> Element {
    let mut anim_handle: Signal<Option<Task>> = use_signal(|| None);
    let current_rect: Signal<Option<RectData>> = use_signal(|| None);

    use_effect(move || {
        if let Some(animation) = &animation.read().animation {
            match animation.state {
                AnimationPlayState::Play => {
                    tracing::info!("some transition");
                    let handle = spawn({
                        let mut current_rect = current_rect.to_owned();
                        let mut animation = animation.to_owned();
                        async move {
                            current_rect.set(Some(animation.from.clone()));
                            while current_rect.read().as_ref().unwrap() != &animation.to {
                                current_rect.set(Some(animation.step().await));
                            }
                            anim_handle.set(None);
                        }
                    });
                    anim_handle.set(Some(handle));
                }
                AnimationPlayState::Pause => {
                    tracing::info!("pausing");
                    anim_handle.write().as_mut().map(|handle| handle.pause());
                }
            }
        }
    });

    let render_state = use_memo(move || {
        current_rect
            .read()
            .as_ref()
            .map_or(String::new(), |rect| rect.to_css())
    });

    rsx! {
        div {
            style: "position: relative; {render_state}",
            {children}
        }
    }
}
