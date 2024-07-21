#![allow(non_snake_case)]

use animatable::{
    components::Animatable,
    controllers::{use_flipbook_signal, AnimationBuilder},
};
use dioxus::prelude::*;
use dioxus_elements::geometry::euclid::{Point2D, Rect, Size2D};
use tracing::Level;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

#[component]
fn App() -> Element {
    let mut animation_controller = use_flipbook_signal();

    let is_resting = use_memo(move || animation_controller.read().read_is_finished());
    rsx! {
        Animatable {
            controller: animation_controller,
            div {
                style: "background-color: red; width: 100%; height: 100%;",
            }
        }
        button {
            // 'stop and drop all' and start this now.
            onclick: move |_| animation_controller.write().play_now(AnimationBuilder::default().animate_to(Rect::new(Point2D::new(0., 0.), Size2D::new(200., 200.,)))),
            "play now"
        }
        button {
            // add to waiting animations
            //onclick: move |_| animation_controller.queue_to_400(),
            "queue to 400"
        }
        button {
            // resume if paused
            onclick: move |_| animation_controller.write().resume(),
            "resume"
        }
        button {
            //pause if playing
            onclick: move |_| animation_controller.write().pause(),
            "pause"
        }
        button {
            // pause. drop current anim & queue. resume
            onclick: move |_| animation_controller.write().drop_all(),
            "stop and drop all"
        }
        p {
            "is resting: {is_resting}",
        }
    }
}
