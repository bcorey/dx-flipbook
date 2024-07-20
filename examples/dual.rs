use animatable::{
    components::Animatable,
    controllers::{AnimationBuilder, AnimationController},
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
    let mut ctrl_1 = use_signal(|| AnimationController::default());
    let mut ctrl_2 = use_signal(|| AnimationController::default());

    let mut play_both = move || {
        let end_1 = Point2D::new(400f64, 0f64);
        let end_2 = Point2D::new(0f64, 400f64);
        let size = Size2D::new(100f64, 100f64);
        let anim_1 = AnimationBuilder::default().animate_to(Rect::new(end_1, size));
        let anim_2 = AnimationBuilder::default().animate_to(Rect::new(end_2, size));
        ctrl_1.write().play_now(anim_1);
        ctrl_2.write().play_now(anim_2);
    };
    rsx! {
        button {
            onclick: move |_| play_both(),
            "play both",
        }

        Animatable {
            controller: ctrl_1,
            div {
                style: "width: 100px; height: 100px; background-color: red;",
            }
        }
        br {}
        Animatable {
            controller: ctrl_2,
            div {
                style: "width: 100px; height: 100px; background-color: green;",
            }
        }
    }
}
