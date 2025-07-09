use std::sync::{Arc, Mutex};

use event::EventHandler;
use object::Object;

use crate::{
    rendering::{EngineRenderer, RendererType},
    utils::{Shared, SharedBox, new_shared},
    windowing::Windower,
};

pub mod component;
pub mod event;
pub mod object;

pub struct Engine {
    pub renderer: Option<EngineRenderer>,
    pub event_handler: Option<EventHandler>,
    pub windower: Option<Windower>,

    pub objects: Vec<SharedBox<dyn Object>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            renderer: None,
            event_handler: None,
            windower: None,
            objects: Vec::new(),
        }
    }

    pub fn into_init(
        self,
        objects: &[SharedBox<dyn Object>],
        _renderer_type: RendererType,
        mut renderer: EngineRenderer,
    ) -> anyhow::Result<Shared<Self>> {
        // let mut renderer = EngineRenderer::new(renderer_type, &self.objects);
        let engine_arc = Arc::new(Mutex::new(self));
        let event_handler = EventHandler::new(Arc::downgrade(&engine_arc));
        let windower = Windower::new_init(Arc::downgrade(&engine_arc));

        let mut engine = engine_arc.lock().expect("mutex lock failed");

        renderer.set_objects(objects);

        renderer.start_render();

        engine.renderer = None;
        engine.event_handler = Some(event_handler);
        engine.windower = Some(windower);
        engine.objects = objects.to_vec();

        Ok(engine_arc.clone())
    }

    pub fn set_objects(&mut self, objects: &[SharedBox<dyn Object>]) {
        self.objects = objects.to_vec();
    }
}
