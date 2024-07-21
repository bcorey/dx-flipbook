#[allow(non_snake_case)]
use dioxus::prelude::*;

use crate::controllers::UseFlipbook;

#[component]
pub fn Animatable(
    controller: Signal<UseFlipbook>,
    style: Option<String>,
    children: Element,
) -> Element {
    let render_state = use_memo(move || {
        let mut state = controller.read().read_render_state();
        if let Some(style) = &style {
            state = format!("{} {}", style, state);
        }
        state
    });
    rsx! {
        div {
            style: "display: flex; position: absolute; {render_state}",
            onmounted: move |cx| controller.write().set_mounted_data(cx.data()),
            {children}
        }
    }
}
