mod three_d_renderer;

use std::sync::{Arc, Mutex};

use three_d_renderer::ThreedRenderer;

use crate::{engine::object::Object, utils::SharedBox};

/// trait for renderers, not really used yet
pub trait Renderer {
    fn start_render(self) -> anyhow::Result<()>;
    fn set_objects(&mut self, objects: &[SharedBox<dyn Object>]);
}

/// basic renderer abstraction
pub struct EngineRenderer {
    pub objects: Vec<SharedBox<dyn Object>>,
    pub renderer: ThreedRenderer,
}

impl EngineRenderer {
    /// create new EngineRenderer
    pub fn new(objects: &[SharedBox<dyn Object>]) -> Self {
        Self {
            objects: objects.to_vec(),
            renderer: ThreedRenderer::new(objects),
        }
    }

    /// sets objects to render
    pub fn set_objects(&mut self, objects: &[SharedBox<dyn Object>]) {
        self.objects = objects.to_vec();
        self.renderer.set_objects(objects);
    }

    /// starts renderer
    pub fn start_render(mut self) -> anyhow::Result<()> {
        self.renderer.start_render()
    }
}
