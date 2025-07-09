mod three_d_renderer;

use std::sync::{Arc, Mutex};

use three_d_renderer::ThreedRenderer;

use crate::{engine::object::Object, utils::SharedBox};

/// trait for renderers, not really used yet
pub trait Renderer {
    fn start_render(self) -> anyhow::Result<()>;
    fn set_objects(&mut self, objects: &[SharedBox<dyn Object>]);
}

pub enum RendererType {
    ThreeD,
}

/// basic renderer abstraction
pub struct EngineRenderer {
    pub objects: Vec<SharedBox<dyn Object>>,
    pub renderer: ThreedRenderer,
}

impl EngineRenderer {
    /// create new EngineRenderer
    pub fn new(renderer_type: RendererType, objects: &[SharedBox<dyn Object>]) -> Self {
        let renderer = match renderer_type {
            RendererType::ThreeD => ThreedRenderer::new(objects),
        };
        Self {
            objects: objects.to_vec(),
            renderer,
        }
    }

    /// sets objects to render
    pub fn set_objects(&mut self, objects: &[SharedBox<dyn Object>]) {
        self.objects = objects.to_vec();
        self.renderer.set_objects(objects);
    }

    /// starts renderer
    pub fn start_render(self) -> anyhow::Result<()> {
        self.renderer.start_render()
    }
}
