use std::sync::{Mutex, Weak};

use winit::{
    event_loop,
    window::{self, Window},
};

use crate::engine::Engine;

use crate::utils::WeakShared;

mod app;

pub struct Windower {
    event_loop: winit::event_loop::EventLoop<()>,

    engine: WeakShared<Engine>,
}

impl Windower {
    pub fn new_init(engine: WeakShared<Engine>) -> Self {
        let event_loop = event_loop::EventLoop::new().unwrap();

        Self { event_loop, engine }
    }
}
