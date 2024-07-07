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
    let mut animation_controller = use_signal(|| AnimationController::default());
    rsx! {
        Animatable {
            controller: animation_controller,
            div {
                style: "background-color: red; width: 100%; height: 100%;",
            }
        }
        button {
            onclick: move |_| animation_controller.write().to_400(),
            "animate"
        }
        button {
            onclick: move |_| animation_controller.write().abort(),
            "abort"
        }
        button {
            onclick: move |_| animation_controller.write().play(),
            "play"
        }
        button {
            onclick: move |_| animation_controller.write().pause(),
            "pause"
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
struct Stopwatch {
    lap_start: Option<web_time::SystemTime>,
    elapsed: web_time::Duration,
}

impl Stopwatch {
    fn new() -> Self {
        Self {
            lap_start: None,
            elapsed: web_time::Duration::ZERO,
        }
    }

    fn stop(&mut self) {
        self.lap_start = None;
    }

    fn start(&mut self) {
        self.lap_start = Some(web_time::SystemTime::now());
    }

    fn lap(&mut self) {
        if let Some(lap_start) = self.lap_start {
            let lap_elapsed = lap_start.elapsed().unwrap();
            self.elapsed = self.elapsed.checked_add(lap_elapsed).unwrap();
            self.start();
        }
    }

    fn get_elapsed(&mut self) -> web_time::Duration {
        self.lap();
        self.elapsed
    }

    fn clear(&mut self) {
        self.stop();
        self.elapsed = web_time::Duration::ZERO;
    }
}

#[derive(Clone, PartialEq, Debug)]
struct AnimationTransition {
    from: RectData,
    to: RectData,
    start_delay: Option<web_time::Duration>,
    duration: web_time::Duration,
    min_frame_duration: web_time::Duration,
    linear_progress: f32,
}

impl AnimationTransition {
    fn new(from: RectData, to: RectData) -> Self {
        let min_frame_duration = Self::get_frame_duration_from_refresh_rate(MAX_RATE_60HZ);
        Self {
            from,
            to,
            start_delay: Some(web_time::Duration::from_millis(1000u64)),
            duration: web_time::Duration::from_millis(1000u64),
            min_frame_duration,
            linear_progress: 0f32,
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

    async fn step(&mut self, total_elapsed: web_time::Duration) -> RectData {
        let frame_start = web_time::SystemTime::now();
        tracing::info!("total elapsed time: {:?}", total_elapsed);
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
struct AnimationController {
    animation: Option<AnimationTransition>,
    command: AnimationCommand,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self {
            animation: None,
            command: AnimationCommand::Play,
        }
    }
}

impl AnimationController {
    // fn animate_to(&mut self, rect: RectData) {
    //     let from =
    //     self.animation = Some(AnimationTransition::new( rect))
    // }

    fn to_400(&mut self) {
        self.animation = Some(AnimationTransition::move_x_linear());
    }

    fn abort(&mut self) {
        self.command = AnimationCommand::Abort;
    }

    fn play(&mut self) {
        self.command = AnimationCommand::Play;
    }

    fn pause(&mut self) {
        self.command = AnimationCommand::Pause;
    }
}

#[derive(Clone, PartialEq, Debug)]
enum AnimationCommand {
    Play,
    Pause,
    Abort,
}

#[component]
fn Animatable(controller: Signal<AnimationController>, children: Element) -> Element {
    let mut anim_handle: Signal<Option<Task>> = use_signal(|| None);
    let mut transition: Signal<Option<AnimationTransition>> = use_signal(|| None);
    let mut stopwatch = use_signal(|| Stopwatch::new());
    let mut delay_stopwatch = use_signal(|| Stopwatch::new());
    let current_rect: Signal<Option<RectData>> = use_signal(|| None);

    use_effect(move || match controller.read().command {
        AnimationCommand::Play => {
            if let Some(anim) = &controller.read().animation {
                if transition.read().is_none() {
                    transition.set(Some(anim.clone()));
                    if let Some(mut current_transition) = transition() {
                        let handle = spawn({
                            let mut current_rect = current_rect.to_owned();
                            async move {
                                if let Some(start_delay) = &current_transition.start_delay {
                                    tracing::info!("SOME DELAY");
                                    delay_stopwatch.write().start();
                                    let delay = gloo_timers::future::sleep(*start_delay);
                                    delay.await;
                                }
                                stopwatch.write().start();
                                current_rect.set(Some(current_transition.from.clone()));
                                while current_rect.read().as_ref().unwrap()
                                    != &current_transition.to
                                {
                                    let elapsed = stopwatch.write().get_elapsed();
                                    current_rect.set(Some(current_transition.step(elapsed).await));
                                }
                                anim_handle.set(None);
                                transition.set(None);
                                stopwatch.write().clear();
                                delay_stopwatch.write().clear();
                                tracing::warn!("ENDED TRANSITION");
                            }
                        });
                        anim_handle.set(Some(handle));
                    }
                } else {
                    if let Some(active_transition) = transition() {
                        anim_handle.write().as_mut().map(|handle| handle.resume());
                        stopwatch.write().start();
                    }
                }
            }
        }
        AnimationCommand::Pause => {
            stopwatch.write().stop(); // don't count pause duration as elapsed animation time
            anim_handle.write().as_mut().map(|handle| handle.pause()); // stop polling loop
        }
        AnimationCommand::Abort => {
            if let Some(delay) = transition().map(|anim| anim.start_delay.clone()).flatten() {
                let elapsed = delay_stopwatch.write().get_elapsed();
                if elapsed < delay {
                    anim_handle.write().as_mut().map(|handle| {
                        handle.pause();
                        handle.cancel();
                    }); // stop polling loop
                    anim_handle.set(None);
                    transition.set(None);
                }
                delay_stopwatch.write().clear();
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
