#[allow(non_snake_case)]
use dioxus::prelude::*;

use crate::{
    controllers::{AnimationCommand, AnimationController, AnimationTransition},
    rectdata::RectData,
    stopwatch::Stopwatch,
};

#[component]
pub fn Animatable(controller: Signal<AnimationController>, children: Element) -> Element {
    let mut anim_handle: Signal<Option<Task>> = use_signal(|| None);
    let mut transition: Signal<Option<AnimationTransition>> = use_signal(|| None);
    let mut stopwatch = use_signal(|| Stopwatch::new());
    let mut delay_stopwatch = use_signal(|| Stopwatch::new());
    let mut current_rect: Signal<Option<RectData>> = use_signal(|| None);

    let mut spawn_animation = move |mut current_transition: AnimationTransition| {
        let handle = spawn(async move {
            if let Some(start_delay) = &current_transition.start_delay {
                delay_stopwatch.write().start();
                let delay = gloo_timers::future::sleep(*start_delay);
                delay.await;
            }

            stopwatch.write().start();
            current_rect.set(Some(current_transition.from.clone()));
            while !current_transition.is_finished() {
                let elapsed = stopwatch.write().get_elapsed();
                current_rect.set(Some(current_transition.step(elapsed).await));
            }
            current_rect.set(Some(current_transition.to));
            //cleanup
            anim_handle.set(None);
            //transition.set(None);
            stopwatch.write().clear();
            delay_stopwatch.write().clear();
            controller.write().command = AnimationCommand::Rest;
        });
        anim_handle.set(Some(handle));
    };

    let mut clear_hooks = move || {
        anim_handle.write().as_mut().map(|handle| handle.cancel());
        anim_handle.set(None);

        transition.set(None);
        stopwatch.write().clear();
        delay_stopwatch.write().clear();
    };

    use_effect(move || match controller().command {
        AnimationCommand::Play => {
            if let Some(anim) = &controller.read().animation {
                if transition.read().is_none() {
                    transition.set(Some(anim.clone()));
                    spawn_animation(anim.clone());
                    tracing::info!("command: play: new animation");
                } else {
                    tracing::info!("command: play: paused animation");

                    if let Some(_animation) = transition() {
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
        AnimationCommand::Reverse => {
            if let Some(animation) = transition() {
                let from = current_rect.read().as_ref().unwrap().clone();
                let to = animation.from;
                let duration = web_time::Duration::from_millis(1000);
                controller
                    .write()
                    .animate(from, to, duration, animation.easing);
                controller.write().command = AnimationCommand::Play;
                anim_handle.write().as_mut().map(|handle| handle.cancel());
                spawn_animation(controller.read().animation.as_ref().unwrap().clone());
                tracing::info!("command: reverse");
            }
        }
        AnimationCommand::Rest => clear_hooks(),
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
