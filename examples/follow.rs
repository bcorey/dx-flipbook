#![allow(non_snake_case)]

use animatable::{
    components::Animatable,
    controllers::{AnimationBuilder, AnimationController},
    easing::Easing,
    rectdata::RectData,
};
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
    let ball_size = use_signal(|| 100f64);
    let mut target_position = use_signal(|| None as Option<RectData>);

    let mut easing_style = use_signal(|| Easing::BackOut);

    let mut mark_target = move |evt: PointerEvent| {
        let coords = evt.data.client_coordinates();
        let mut to = RectData::new(coords.x, coords.y, ball_size(), ball_size());
        target_position.set(Some(to.clone()));
        to.position.x -= ball_size() / 2.;
        to.position.y -= ball_size() / 2.;
        let anim_builder = AnimationBuilder::default()
            .animate_to(to)
            .with_easing(easing_style());
        animation_controller.write().queue(anim_builder);
    };

    let target_css = use_memo(move || {
        target_position
            .read()
            .as_ref()
            .map_or(String::new(), |rect| {
                format!("left: {}px; top: {}px;", rect.position.x, rect.position.y)
            })
    });
    rsx! {
        div {
            style: "width: 100%; height: 100vh;",
            onpointerdown: move |evt| mark_target(evt),
            Animatable {
                controller: animation_controller,
                div {
                    style: "background-color: red; width: 100%; height: 100%; border-radius: 100%;",
                }
            }
            div {
                style: "position: absolute; background-color: blue; width: 20px; height: 20px; border-radius: 100%; {target_css}",
            }

            button {
                onclick: move |_| easing_style.set(Easing::ElasticOut),
                "Elastic Out",
            }
            button {
                onclick: move |_| easing_style.set(Easing::BackOut),
                "Back Out",
            }
            button {
                onclick: move |_| easing_style.set(Easing::Linear),
                "Linear",
            }
        }
    }
}
