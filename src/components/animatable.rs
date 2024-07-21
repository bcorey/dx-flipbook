use std::{collections::VecDeque, rc::Rc};

#[allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_elements::geometry::euclid::Rect;

use crate::{
    controllers::{AnimationBuilder, AnimationCommand, AnimationController, AnimationTransition},
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
        tracing::info!("play now from queue");
        self.drop_all();
        self.push(anim);
    }

    pub fn push(&mut self, anim: AnimationBuilder) {
        self.queue.push_back(anim);
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<AnimationBuilder> {
        self.queue.pop_front()
    }

    pub fn size(&self) -> usize {
        self.queue.len()
    }
}

#[component]
pub fn Animatable(
    controller: Signal<AnimationController>,
    style: Option<String>,
    children: Element,
) -> Element {
    let mut anim_handle: Signal<Option<Task>> = use_signal(|| None);
    let mut stopwatch = use_signal(|| Stopwatch::new());
    let mut current_rect: Signal<Option<Rect<f64, f64>>> = use_signal(|| None);

    let mut queue = use_signal(|| AnimationQueue::new());
    let controller_rect_is_stale =
        use_memo(move || current_rect.read().as_ref() != controller.peek().get_rect().as_ref());
    use_effect(move || {
        let stale = controller_rect_is_stale();
        if !stale {
            return;
        }
        if let Some(cur_rect) = current_rect.peek().clone() {
            controller.write().private_set_rect(cur_rect);
        }
    });

    let mut spawn_animation = move |mut current_transition: AnimationTransition| {
        let handle = spawn(async move {
            controller.write().set_busy();
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
            controller.write().set_resting();
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

    let mut parse_queue = move || {
        tracing::info!("evaluating queue: {:?}", queue.peek());

        if let Some(anim_builder) = queue.write().pop_front() {
            match (anim_builder.from.clone(), anim_builder.to.clone()) {
                (_, None) => spawn_delay(anim_builder.duration),
                (None, Some(to)) => {
                    let from = current_rect.peek().as_ref().unwrap().clone();
                    if from == to {
                        tracing::error!("requested animation has same origin and destination");
                        return;
                    }
                    let animation = AnimationTransition::new(anim_builder, from, to);
                    spawn_animation(animation);
                }
                (Some(from), Some(to)) => {
                    let animation = AnimationTransition::new(anim_builder, from, to);
                    spawn_animation(animation);
                }
            }
        }
    };

    use_effect(move || {
        // subscribe to anim handle and play the next animation in queue when the current one is done
        tracing::info!("reading queue and handle");
        let queue_read = queue.read().clone();
        if anim_handle.read().is_some() {
            tracing::info!("handle is empty");
            return;
        }
        if queue_read.is_empty() {
            tracing::info!("queue is empty");
            return;
        }
        parse_queue();
    });

    let mut clear_hooks = move || {
        anim_handle.write().as_mut().map(|handle| handle.cancel());
        anim_handle.set(None);
        stopwatch.write().clear();
    };

    use_effect(move || {
        let cmd = controller().get_command();
        tracing::info!("processing command {:?}", cmd);
        match cmd {
            AnimationCommand::Resume => {
                tracing::info!("command: play: paused animation");

                if let Some(handle) = anim_handle() {
                    if handle.paused() {
                        handle.resume();
                        stopwatch.write().start();
                    }
                }
                controller.write().clear_command();
            }
            AnimationCommand::Pause => {
                stopwatch.write().stop(); // don't count pause duration as elapsed animation time
                anim_handle.write().as_mut().map(|handle| handle.pause()); // stop polling loop
                controller.write().clear_command();
            }
            AnimationCommand::DropAll => {
                clear_hooks();
                queue.write().drop_all();
                controller.write().clear_command();
                controller.write().set_resting();
            }
            AnimationCommand::Queue(anim) => {
                queue.write().push(anim);
                controller.write().clear_command();
            }
            AnimationCommand::PlayNow(anim) => {
                clear_hooks();
                controller.write().set_busy();
                queue.write().play_now(anim);
                tracing::info!("play now!");
                parse_queue();
                controller.write().clear_command();
            }
            AnimationCommand::SetRect(rect) => {
                if anim_handle.peek().is_none() {
                    current_rect.set(Some(rect));
                }
                controller.write().clear_command();
            }
            AnimationCommand::None => {}
        }
    });

    let render_state = use_memo(move || {
        let mut position = current_rect.read().as_ref().map_or(String::new(), |rect| {
            format!(
                "width: {}px; height: {}px; left: {}px; top: {}px;",
                rect.size.width, rect.size.height, rect.origin.x, rect.origin.y
            )
        });
        if let Some(style) = &style {
            position = format!("{} {}", style, position);
        }
        position
    })
    .to_string();

    let set_initial_rect = move |data: Rc<MountedData>| async move {
        let client_rect = data.get_client_rect();

        if let Ok(rect) = client_rect.await {
            current_rect.set(Some(rect));
        }
    };

    rsx! {
        div {
            style: "display: flex; position: absolute; {render_state}",
            onmounted: move |cx| set_initial_rect(cx.data()),
            {children}
        }
    }
}
