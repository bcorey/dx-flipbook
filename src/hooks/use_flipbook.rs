use std::rc::Rc;

use dioxus::{
    dioxus_core::Task,
    hooks::{use_effect, use_signal},
    html::{geometry::euclid::Rect, MountedData},
    prelude::spawn,
    signals::{Readable, Signal, Writable},
};

use crate::{
    controllers::{AnimationBuilder, AnimationQueue, AnimationTransition},
    stopwatch::use_stopwatch_signal,
};

#[derive(Clone, PartialEq, Debug)]
pub enum FlipbookStatus {
    Busy,
    Resting,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FlipbookCommand {
    Resume,
    Pause,
    PlayNow(AnimationBuilder),
    DropAll,
    Queue(AnimationBuilder),
    SetRect(Rect<f64, f64>),
    None,
}

#[derive(Clone, PartialEq)]
pub struct UseFlipbook {
    current_rect: Signal<Option<Rect<f64, f64>>>,
    status: Signal<FlipbookStatus>,
    command: Signal<FlipbookCommand>,
    mounted: Signal<Option<Rc<MountedData>>>,
}

impl UseFlipbook {
    /// position is relative until the rect is read from MountedData.
    /// then the position and size are controlled by the animatable and the position is absolute.
    pub(crate) fn read_render_state(&self) -> String {
        self.current_rect
            .read()
            .as_ref()
            .map_or("position: relative;".to_string(), |rect| {
                format!(
                    "width: {}px; height: {}px; left: {}px; top: {}px;",
                    rect.size.width, rect.size.height, rect.origin.x, rect.origin.y
                )
            })
    }

    pub fn set_mounted_data(&mut self, data: Rc<MountedData>) {
        self.mounted.set(Some(data));
    }

    pub fn peek_rect(&self) -> Option<Rect<f64, f64>> {
        self.current_rect.peek().clone()
    }

    pub fn read_rect(&self) -> Option<Rect<f64, f64>> {
        self.current_rect.read().clone()
    }

    pub fn set_rect(&mut self, rect: Rect<f64, f64>) {
        self.command.set(FlipbookCommand::SetRect(rect));
    }

    pub fn peek_status(&self) -> FlipbookStatus {
        self.status.peek().clone()
    }

    pub fn read_status(&self) -> FlipbookStatus {
        self.status.read().clone()
    }

    pub fn peek_is_finished(&self) -> bool {
        self.status.peek().clone() == FlipbookStatus::Resting
    }

    pub fn read_is_finished(&self) -> bool {
        self.status.read().clone() == FlipbookStatus::Resting
    }

    pub fn queue(&mut self, anim: AnimationBuilder) {
        self.command.set(FlipbookCommand::Queue(anim));
    }

    pub fn play_now(&mut self, anim: AnimationBuilder) {
        self.command.set(FlipbookCommand::PlayNow(anim));
    }

    pub fn resume(&mut self) {
        self.command.set(FlipbookCommand::Resume);
    }

    pub fn pause(&mut self) {
        self.command.set(FlipbookCommand::Pause);
    }

    pub fn drop_all(&mut self) {
        self.command.set(FlipbookCommand::DropAll);
    }
}

fn use_flipbook() -> UseFlipbook {
    let mut current_rect = use_signal(|| None as Option<Rect<f64, f64>>);
    let mut status = use_signal(|| FlipbookStatus::Resting);
    let mut command = use_signal(|| FlipbookCommand::None);

    let mut anim_handle: Signal<Option<Task>> = use_signal(|| None);
    let mut stopwatch = use_stopwatch_signal();

    let mut queue = use_signal(|| AnimationQueue::new());

    let mounted = use_signal(|| None as Option<Rc<MountedData>>);
    let read_mounted = move |data: Option<Rc<MountedData>>| async move {
        let client_rect = data.as_ref().map(|el| el.get_client_rect());

        if let Some(client_rect) = client_rect {
            if let Ok(rect) = client_rect.await {
                tracing::info!("setting rect from mounted data");
                current_rect.set(Some(rect));
            }
        }
    };
    use_effect(move || {
        let read = mounted();
        spawn(async move {
            read_mounted(read).await;
        });
    });

    let mut spawn_animation = move |mut current_transition: AnimationTransition| {
        let handle = spawn(async move {
            status.set(FlipbookStatus::Busy);
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
            status.set(FlipbookStatus::Resting);
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

    let mut clear_hooks = move || {
        anim_handle.write().as_mut().map(|handle| handle.cancel());
        anim_handle.set(None);
        stopwatch.write().clear();
    };

    let mut parse_queue = move || {
        tracing::info!("evaluating queue: {:?}", queue.peek());

        if let Some(anim_builder) = queue.write().pop_front() {
            match (anim_builder.from.clone(), anim_builder.to.clone()) {
                (_, None) => spawn_delay(anim_builder.duration),
                (None, Some(to)) => {
                    let from = current_rect.peek().as_ref().unwrap().clone();
                    if from == to {
                        tracing::error!(
                            "requested animation has same origin and destination: {:?} {:?}",
                            from,
                            to
                        );
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

    use_effect(move || {
        let cmd = command();
        tracing::info!("processing command {:?}", cmd);
        match cmd {
            FlipbookCommand::Resume => {
                tracing::info!("command: play: paused animation");

                if let Some(handle) = anim_handle() {
                    if handle.paused() {
                        handle.resume();
                        stopwatch.write().start();
                    }
                }
                command.set(FlipbookCommand::None);
            }
            FlipbookCommand::Pause => {
                stopwatch.write().stop(); // don't count pause duration as elapsed animation time
                anim_handle.write().as_mut().map(|handle| handle.pause()); // stop polling loop
                command.set(FlipbookCommand::None);
            }
            FlipbookCommand::DropAll => {
                clear_hooks();
                queue.write().drop_all();
                command.set(FlipbookCommand::None);
                status.set(FlipbookStatus::Resting);
            }
            FlipbookCommand::Queue(anim) => {
                queue.write().push(anim);
                command.set(FlipbookCommand::None)
            }
            FlipbookCommand::PlayNow(anim) => {
                clear_hooks();
                status.set(FlipbookStatus::Busy);
                queue.write().play_now(anim);
                tracing::info!("play now!");
                parse_queue();
                command.set(FlipbookCommand::None);
            }
            FlipbookCommand::SetRect(rect) => {
                if anim_handle.peek().is_none() {
                    current_rect.set(Some(rect));
                }
                command.set(FlipbookCommand::None);
            }
            FlipbookCommand::None => {}
        }
    });

    UseFlipbook {
        current_rect,
        status,
        command,
        mounted,
    }
}

pub fn use_flipbook_signal() -> Signal<UseFlipbook> {
    let ctrl = use_flipbook();
    use_signal(|| ctrl)
}
