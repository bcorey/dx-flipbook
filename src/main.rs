#![allow(non_snake_case)]

use animatable::{components::Animatable, controllers::AnimationController};
use dioxus::prelude::*;
use tracing::Level;

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
        button {
            onclick: move |_| animation_controller.write().reverse(),
            "reverse"
        }
        button {
            onclick: move |_| animation_controller.write().rest(),
            "rest"
        }
    }
}
