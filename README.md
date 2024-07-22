## Animation component for Dioxus
This is in early days but will see a crates.io release once the API is finalized. 
- To use, declare a controller with `use_flipbook_signal()` and pass it to the animation component.
- Put any items you wish to see transformed inside the component, and set their bounds to fill the component.
- create an animation with `AnimationBuilder`.
    - You can set any easing type available on https://easings.net/ thanks to the `simple-easing` crate used here.
    - no need to specify a start location, unlike css animations
- use `playNow(animation_builder)` or `queue(animation_builder)` on the controller and sit back and watch the animations

https://github.com/user-attachments/assets/2dac0c31-d6ee-46d1-9be1-a75a6be66089

