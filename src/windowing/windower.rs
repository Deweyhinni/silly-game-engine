use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc, RwLock, Weak,
        mpsc::{Receiver, SyncSender},
    },
};

use winit::{
    application::ApplicationHandler,
    event_loop::EventLoopBuilder,
    window::{Window, WindowAttributes, WindowId},
};

use crate::{
    engine::{
        Engine, EngineCommand,
        event::EventHandlerCommand,
        messages::{Message, MessageCommand, MessageContext, Systems},
    },
    rendering::{Renderer, RendererCommand},
    utils::WeakShared,
};

#[derive(Debug, Clone)]
pub enum WindowerCommand {}

pub struct Windower {
    engine: Engine,
    parent_window_id: Option<WindowId>,
    windows: Arc<RwLock<HashMap<WindowId, Arc<Window>>>>,

    pub parent_window_attributes: WindowAttributes,
}

impl Windower {
    pub fn new(engine: Engine, attributes: WindowAttributes) -> Self {
        Self {
            engine,
            parent_window_id: Option::default(),
            windows: Arc::new(RwLock::new(HashMap::default())),
            parent_window_attributes: attributes,
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let event_loop = EventLoopBuilder::default().build().unwrap();

        event_loop
            .run_app(self)
            .map_err(|e| anyhow::anyhow!("running app failed: {e}"))
    }

    fn get_parent_window(&self) -> Option<(Arc<Window>, WindowId)> {
        Some((
            self.windows
                .read()
                .unwrap()
                .get(&self.parent_window_id?)?
                .clone(),
            self.parent_window_id?,
        ))
    }

    fn get_window(&self, window_id: WindowId) -> Option<Arc<Window>> {
        self.windows.read().unwrap().get(&window_id).cloned()
    }
}

impl ApplicationHandler for Windower {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(self.parent_window_attributes.clone())
                .unwrap(),
        );
        let wid = window.id();
        self.parent_window_id = Some(window.id());
        self.windows.write().unwrap().insert(window.id(), window);
        let windows = self.windows.read().unwrap();
        let window = windows
            .get(&self.parent_window_id.expect("no window id"))
            .expect("no window");
        self.engine
            .renderer
            .renderer
            .init(window, &self.engine.default_camera_id)
            .unwrap();
        window.request_redraw();
        log::info!("resumed");
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        // log::info!("window event: {:?}", event);

        let windows = self.windows.read().unwrap();

        let window = windows
            .get(&window_id)
            .expect("window destroyed while in use");

        match event {
            winit::event::WindowEvent::RedrawRequested => {
                let redraw_msg = Message {
                    from: Systems::Windower,
                    to: Systems::Renderer,
                    context: MessageContext {
                        command: MessageCommand::RendererCommand(RendererCommand::Render(
                            window_id,
                        )),
                    },
                };

                self.engine.handle_render(Arc::clone(window));
                let complete_msg = Message {
                    from: Systems::Windower,
                    to: Systems::Engine,
                    context: MessageContext {
                        command: MessageCommand::EngineCommand(EngineCommand::RedrawComplete(
                            window_id,
                        )),
                    },
                };
            }
            winit::event::WindowEvent::Resized(_) => {
                let msg = Message {
                    from: Systems::Windower,
                    to: Systems::Renderer,
                    context: MessageContext {
                        command: MessageCommand::RendererCommand(RendererCommand::HandleResize((
                            window_id,
                            event.clone(),
                        ))),
                    },
                };

                match self
                    .engine
                    .renderer
                    .renderer
                    .handle_resize(Arc::clone(window), &event)
                {
                    Ok(()) => (),
                    Err(e) => {
                        log::error!("handling resize failed: {e}");
                    }
                };
            }
            winit::event::WindowEvent::CloseRequested => {
                let msg = Message {
                    from: Systems::Windower,
                    to: Systems::Renderer,
                    context: MessageContext {
                        command: MessageCommand::RendererCommand(RendererCommand::HandleClose((
                            window_id,
                            event.clone(),
                        ))),
                    },
                };
                log::info!("close requested");
                self.engine
                    .renderer
                    .renderer
                    .handle_close(Arc::clone(window), &event)
                    .unwrap();
                self.windows.write().unwrap().clear();
                event_loop.exit();
            }
            e => {
                let msg = Message {
                    from: Systems::Windower,
                    to: Systems::EventHandler,
                    context: MessageContext {
                        command: MessageCommand::EventHandlerCommand(
                            EventHandlerCommand::WindowEvent((window_id, e.clone())),
                        ),
                    },
                };

                self.engine.event_handler.send_event(window_id, e);
            }
        }
    }
}
