use std::collections::HashMap;

use winit::{
    application::{self, ApplicationHandler},
    dpi::{Position, Size},
    window::{Window, WindowAttributes, WindowId},
};

use crate::{engine::Engine, utils::WeakShared};

#[derive(Debug)]
pub struct WinitApp {
    parent_window_id: Option<WindowId>,
    windows: HashMap<WindowId, Window>,

    pub parent_window_attributes: WindowAttributes,

    engine: WeakShared<Engine>,
}

impl WinitApp {
    pub fn new(attributes: WindowAttributes, engine: WeakShared<Engine>) -> Self {
        Self {
            parent_window_id: Option::default(),
            windows: HashMap::default(),
            parent_window_attributes: attributes,

            engine,
        }
    }
}

impl ApplicationHandler for WinitApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(self.parent_window_attributes.clone())
            .unwrap();
        self.parent_window_id = Some(window.id());
        self.windows.insert(window.id(), window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        let engine = self.engine.upgrade().expect("engine ref not valid").clone();
        match &engine.lock().unwrap().event_handler {
            Some(eh) => {
                eh.send_event(window_id, event);
            }
            None => {
                panic!("no event handler found");
            }
        }
    }
}
