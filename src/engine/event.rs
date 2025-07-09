use winit::{event::WindowEvent, window::WindowId};

use crate::utils::WeakShared;

use super::Engine;

pub struct EventHandler {
    engine: WeakShared<Engine>,
}

impl EventHandler {
    pub fn new(engine: WeakShared<Engine>) -> Self {
        Self { engine }
    }

    pub fn send_event(&self, window_id: WindowId, event: WindowEvent) -> () {
        todo!("idrk event stuff")
    }
}
