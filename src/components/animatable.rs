use std::{collections::VecDeque, rc::Rc};

#[allow(non_snake_case)]
use dioxus::prelude::*;

use crate::{
    controllers::{AnimationBuilder, AnimationCommand, AnimationController, AnimationTransition},
    rectdata::RectData,
    stopwatch::Stopwatch,
};

#[derive(Clone, PartialEq, Debug)]
pub struct AnimationQueue {
    queue: VecDeque<AnimationBuilder>,
}

impl AnimationQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn drop_all(&mut self) {
        self.queue.clear();
    }

    pub fn play_now(&mut self, anim: AnimationBuilder) {
        self.drop_all();
        self.push(anim);
    }

    pub fn push(&mut self, anim: AnimationBuilder) {
        self.queue.push_back(anim);
    }

    pub fn pop_front(&mut self) -> Option<AnimationBuilder> {
        self.queue.pop_front()
    }

    pub fn size(&self) -> usize {
        self.queue.len()
    }
}

#[component]
pub fn Animatable(controller: Signal<AnimationController>, children: Element) -> Element {
    let mut anim_handle: Signal<Option<Task>> = use_signal(|| None);
    let mut stopwatch = use_signal(|| Stopwatch::new());
    let mut current_rect: Signal<Option<RectData>> = use_signal(|| None);

    let mut queue = use_signal(|| AnimationQueue::new());

    let mut spawn_animation = move |mut current_transition: AnimationTransition| {
        let handle = spawn(async move {
            stopwatch.write().start();
            current_rect.set(Some(current_transition.from.clone()));
            while !current_transition.is_finished() {
                let elapsed = stopwatch.write().get_elapsed();
                current_rect.set(Some(current_transition.step(elapsed).await));
            }
            current_rect.set(Some(current_transition.to));
            //cleanup
            anim_handle.set(None);
            stopwatch.write().clear();
        });
        anim_handle.set(Some(handle));
    };

    let mut spawn_delay = move |duration: web_time::Duration| {
        let handle = spawn(async move {
            gloo_timers::future::sleep(duration).await;
            anim_handle.set(None);
        });
        anim_handle.set(Some(handle));
    };

    use_effect(move || {
        // subscribe to anim handle and play the next animation in queue when the current one is done
        if anim_handle.read().is_some() {
            return;
        }
        let _trigger = queue.read().clone();
        tracing::info!("evaluating queue: {:?}", _trigger);
        if let Some(anim_builder) = queue.write().pop_front() {
            match (anim_builder.from.clone(), anim_builder.to.clone()) {
                (_, None) => spawn_delay(anim_builder.duration),
                (None, Some(to)) => {
                    let from = current_rect.peek().as_ref().unwrap().clone();
                    let animation = AnimationTransition::new(anim_builder, from, to);
                    spawn_animation(animation);
                }
                (Some(from), Some(to)) => {
                    let animation = AnimationTransition::new(anim_builder, from, to);
                    spawn_animation(animation);
                }
            }
        }
    });

    let mut clear_hooks = move || {
        anim_handle.write().as_mut().map(|handle| handle.cancel());
        anim_handle.set(None);
        stopwatch.write().clear();
    };

    use_effect(move || match controller().command {
        AnimationCommand::Resume => {
            tracing::info!("command: play: paused animation");

            if let Some(handle) = anim_handle() {
                if handle.paused() {
                    handle.resume();
                    stopwatch.write().start();
                }
            }
            controller.write().command = AnimationCommand::None;
        }
        AnimationCommand::Pause => {
            stopwatch.write().stop(); // don't count pause duration as elapsed animation time
            anim_handle.write().as_mut().map(|handle| handle.pause()); // stop polling loop
            controller.write().command = AnimationCommand::None;
        }
        AnimationCommand::DropAll => {
            clear_hooks();
            queue.write().drop_all();
            controller.write().command = AnimationCommand::None;
        }
        AnimationCommand::Queue(anim) => {
            queue.write().push(anim);
            controller.write().command = AnimationCommand::None;
        }
        AnimationCommand::PlayNow(anim) => {
            clear_hooks();
            queue.write().play_now(anim);
            controller.write().command = AnimationCommand::None;
        }
        AnimationCommand::None => {
            tracing::info!("no command");
        }
    });

    let render_state = use_memo(move || {
        current_rect
            .read()
            .as_ref()
            .map_or(String::new(), |rect| rect.to_css())
    });

    // let div_element = use_signal(|| None as Option<Rc<MountedData>>);

    let set_initial_rect = move |data: Rc<MountedData>| async move {
        let client_rect = data.get_client_rect();

        if let Ok(rect) = client_rect.await {
            let converted_data = RectData::new(
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
            );
            current_rect.set(Some(converted_data));
        }
    };

    rsx! {
        div {
            style: "display: flex; position: relative; {render_state}",
            onmounted: move |cx| set_initial_rect(cx.data()),
            {children}
        }
    }
}
