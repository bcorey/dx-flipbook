use crate::{easing::Easing, rectdata::RectData};

use super::AnimationTransition;

#[derive(Clone, PartialEq, Debug)]
pub struct AnimationController {
    pub animation: Option<AnimationTransition>,
    pub command: AnimationCommand,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self {
            animation: None,
            command: AnimationCommand::Rest,
        }
    }
}

impl AnimationController {
    // fn animate_to(&mut self, rect: RectData) {
    //     let from =
    //     self.animation = Some(AnimationTransition::new( rect))
    // }

    pub fn animate(
        &mut self,
        from: RectData,
        to: RectData,
        duration: web_time::Duration,
        easing: Easing,
    ) {
        self.animation = Some(AnimationTransition::new(from, to, duration, easing));
    }

    pub fn to_400(&mut self) {
        self.animation = Some(AnimationTransition::move_x_linear());
    }

    pub fn abort(&mut self) {
        self.command = AnimationCommand::Abort;
    }

    pub fn play(&mut self) {
        self.command = AnimationCommand::Play;
    }

    pub fn pause(&mut self) {
        self.command = AnimationCommand::Pause;
    }

    pub fn reverse(&mut self) {
        self.command = AnimationCommand::Reverse;
    }

    pub fn rest(&mut self) {
        self.command = AnimationCommand::Rest;
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum AnimationCommand {
    Play,
    Pause,
    Abort,
    Reverse,
    Rest,
}
