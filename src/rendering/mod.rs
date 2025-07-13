mod three_d_renderer;

use std::{
    collections::VecDeque,
    rc::Rc,
    sync::{Arc, Mutex, Weak},
};

use three_d_renderer::ThreedRenderer;
use winit::{
    event::WindowEvent,
    window::{Window, WindowId},
};

use crate::{
    engine::{
        Engine,
        entity::{Entity, EntityRegistry},
        messages::Message,
    },
    utils::{SharedBox, WeakShared},
};

/// trait for renderers, not really used yet
pub trait Renderer {
    // fn start_render(self) -> anyhow::Result<()>;
    fn render(&mut self, window: Arc<Window>) -> anyhow::Result<()>;
    fn handle_resize(&mut self, window: Arc<Window>, event: &WindowEvent) -> anyhow::Result<()>;
    fn handle_scale_factor_change(
        &mut self,
        window: Arc<Window>,
        event: &WindowEvent,
    ) -> anyhow::Result<()>;
    fn handle_close(&mut self, window: Arc<Window>, event: &WindowEvent) -> anyhow::Result<()>;
    fn set_objects(&mut self, objects: EntityRegistry);

    fn get_messages(&self) -> &VecDeque<Message>;
    fn get_messages_mut(&mut self) -> &mut VecDeque<Message>;
    fn clear_messages(&mut self);
}

#[derive(Debug, Clone)]
pub enum RendererCommand {
    Render(WindowId),
    HandleResize((WindowId, WindowEvent)),
    HandleScaleChange((WindowId, WindowEvent)),
    HandleClose((WindowId, WindowEvent)),
}

#[derive(Debug, Clone)]
pub enum RendererType {
    ThreeD,
}

/// basic renderer abstraction
pub struct EngineRenderer {
    pub objects: EntityRegistry,
    pub renderer: ThreedRenderer,
}

impl EngineRenderer {
    /// create new EngineRenderer
    pub fn new(renderer_type: RendererType, objects: EntityRegistry) -> Self {
        let renderer = match renderer_type {
            RendererType::ThreeD => ThreedRenderer::new(objects.clone()),
        };
        Self { objects, renderer }
    }

    /// sets objects to render
    pub fn set_objects(&mut self, objects: EntityRegistry) {
        self.objects = objects.clone();
        self.renderer.set_objects(objects);
    }

    /// renders frame
    pub fn render(&mut self, window: Arc<Window>) -> anyhow::Result<()> {
        self.renderer.render(window)
    }

    pub fn get_messages(&self) -> &VecDeque<Message> {
        self.renderer.get_messages()
    }

    pub fn get_messages_mut(&mut self) -> &mut VecDeque<Message> {
        self.renderer.get_messages_mut()
    }

    pub fn clear_messages(&mut self) {
        self.renderer.clear_messages();
    }
}
