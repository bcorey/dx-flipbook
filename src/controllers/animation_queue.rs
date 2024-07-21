use std::collections::VecDeque;

use super::AnimationBuilder;

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
