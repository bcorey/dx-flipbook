#![allow(non_snake_case)]

use std::thread::current;

use dioxus::prelude::*;
use dioxus_sdk::utils::timing::use_interval;
use futures_util::StreamExt;
use tracing::Level;
use web_time::Duration;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

#[component]
fn App() -> Element {
    let mut animatable = use_context_provider(|| Signal::new(AnimatableState::default()));
    rsx! {
        Animatable {
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

#[derive(Clone, PartialEq, Debug)]
struct AnimationTransition {
    from: RectData,
    to: RectData,
    current: RectData,
    start_time: web_time::SystemTime,
    state: AnimationPlayState,
}

impl AnimationTransition {
    fn new(from: RectData, to: RectData) -> Self {
        let current = from.clone();

        Self {
            from,
            to,
            current,
            start_time: web_time::SystemTime::now(),
            state: AnimationPlayState::Play,
        }
    }

    async fn step(&self) -> RectData {
        let frame_start = web_time::SystemTime::now();
        let max_time = web_time::Duration::from_millis(500u64);
        let elapsed = self
            .start_time
            .elapsed()
            .expect("could not get elapsed time since animation start");

        tracing::info!("stepped");
        if elapsed >= max_time {
            return self.to.clone();
        }

        let percent = elapsed.as_secs_f64() / max_time.as_secs_f64();
        let mut current_rect = self.from.clone();
        let x_diff = self.to.position.x - self.from.position.x;
        current_rect.position.x += x_diff * percent;

        let frame_duration = frame_start
            .elapsed()
            .expect("couldn't get elapsed time during frame");

        let min_frame_duration = Duration::from_millis(8);
        if frame_duration < min_frame_duration {
            let delay_duration = min_frame_duration.as_millis() - frame_duration.as_millis();
            let delay = gloo_timers::future::sleep(Duration::from_millis(delay_duration as u64));
            delay.await;
        }

        current_rect
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
        let from = RectData::new(0f64, 0f64, 200f64, 200f64);
        let to = RectData::new(400f64, 0f64, 200f64, 200f64);
        self.animation = Some(AnimationTransition::new(from, to));
    }

    fn toggle_play_state(&mut self) {
        self.animation
            .as_mut()
            .map(|animation| animation.state = animation.state.toggle());
    }

    fn render_data(&self) -> String {
        match &self.animation {
            Some(animation) => animation.current.to_css(),
            None => String::new(),
        }
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

enum Action {
    Play,
    Pause,
}

#[component]
fn Animatable(children: Element) -> Element {
    let state = use_context::<Signal<AnimatableState>>();
    let anim_handle: Signal<Option<Task>> = use_signal(|| None);
    let current_rect: Signal<Option<RectData>> = use_signal(|| None);
    let animation_handle = use_coroutine(|mut rx: UnboundedReceiver<AnimationPlayState>| {
        let state = state.to_owned();
        let mut anim_handle = anim_handle.to_owned();
        async move {
            while let Some(action) = rx.next().await {
                match action {
                    AnimationPlayState::Play => {
                        if let Some(transition) = state.read().animation.clone() {
                            tracing::info!("some transition");
                            let handle = spawn({
                                let mut current_rect = current_rect.to_owned();
                                async move {
                                    current_rect.set(Some(transition.from.clone()));
                                    while current_rect.read().as_ref().unwrap() != &transition.to {
                                        current_rect.set(Some(transition.step().await));
                                    }
                                    anim_handle.set(None);
                                }
                            });
                            anim_handle.set(Some(handle));
                        }
                    }
                    AnimationPlayState::Pause => {
                        tracing::info!("pausing");
                        anim_handle.write().as_mut().map(|handle| handle.pause());
                    }
                }
            }
        }
    });

    use_effect(move || {
        if let Some(animation) = &state.read().animation {
            animation_handle.send(animation.state.clone());
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
