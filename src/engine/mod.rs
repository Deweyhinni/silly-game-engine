use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use event::{EventHandler, EventHandlerCommand};
use messages::{Message, MessageCommand};
use object::Object;
use winit::{
    event_loop::EventLoop,
    platform::run_on_demand::EventLoopExtRunOnDemand,
    window::{Window, WindowId},
};

use crate::{
    rendering::{EngineRenderer, Renderer, RendererCommand, RendererType},
    utils::{Shared, SharedBox},
};

pub mod component;
pub mod event;
pub mod messages;
pub mod object;

#[derive(Debug, Clone)]
pub enum EngineCommand {
    RedrawComplete(WindowId),
}

pub struct Engine {
    pub renderer: EngineRenderer,
    pub event_handler: EventHandler,

    windows: Arc<RwLock<HashMap<WindowId, Arc<Window>>>>,
    pub objects: Vec<SharedBox<dyn Object>>,
}

impl Engine {
    pub fn new(renderer_type: RendererType, objects: &[SharedBox<dyn Object>]) -> Self {
        Self {
            renderer: EngineRenderer::new(renderer_type, objects),
            event_handler: EventHandler::new(),
            windows: Arc::new(RwLock::new(HashMap::new())),
            objects: objects.to_vec(),
        }
    }

    pub fn init(
        &mut self,
        windows: &Arc<RwLock<HashMap<WindowId, Arc<Window>>>>,
    ) -> anyhow::Result<()> {
        self.handle_messages();

        self.windows = Arc::clone(&windows);

        Ok(())
    }

    /// handles the rendering of a frame
    pub fn handle_render(&mut self, window: Arc<Window>) {
        self.renderer.render(window).unwrap();
    }

    pub fn handle_messages(&mut self) {
        let mut msg_queues = [
            self.event_handler.get_messages().clone(),
            self.renderer.get_messages().clone(),
        ];

        self.event_handler.clear_messages();
        self.renderer.clear_messages();

        log::info!("messages: {:?}", msg_queues);

        for queue in msg_queues.iter_mut() {
            while queue.len() > 0 {
                let msg = match queue.pop_front() {
                    Some(m) => m,
                    None => {
                        log::error!("message deque failed");
                        continue;
                    }
                };
                log::info!("message: {:?}", msg);
                match self.handle_message(msg) {
                    Ok(()) => (),
                    Err(e) => {
                        log::error!("error: {:?}", e);
                        continue;
                    }
                };
            }
        }
    }

    fn handle_message(&mut self, msg: Message) -> anyhow::Result<()> {
        match msg.context.command {
            MessageCommand::RendererCommand(rc) => match rc {
                RendererCommand::Render(wid) => {
                    log::info!("render message");
                    self.renderer.render(Arc::clone(
                        self.windows
                            .read()
                            .unwrap()
                            .get(&wid)
                            .ok_or(anyhow::anyhow!("window not found"))?,
                    ))
                }
                RendererCommand::HandleResize((wid, wevent)) => {
                    self.renderer.renderer.handle_resize(
                        Arc::clone(
                            self.windows
                                .read()
                                .unwrap()
                                .get(&wid)
                                .ok_or(anyhow::anyhow!("window not found"))?,
                        ),
                        &wevent,
                    )
                }
                RendererCommand::HandleScaleChange((wid, wevent)) => {
                    self.renderer.renderer.handle_scale_factor_change(
                        Arc::clone(
                            self.windows
                                .read()
                                .unwrap()
                                .get(&wid)
                                .ok_or(anyhow::anyhow!("window not found"))?,
                        ),
                        &wevent,
                    )
                }
                RendererCommand::HandleClose((wid, wevent)) => self.renderer.renderer.handle_close(
                    Arc::clone(
                        self.windows
                            .read()
                            .unwrap()
                            .get(&wid)
                            .ok_or(anyhow::anyhow!("window not found"))?,
                    ),
                    &wevent,
                ),
            },
            MessageCommand::EventHandlerCommand(ehc) => match ehc {
                EventHandlerCommand::WindowEvent((wid, wevent)) => {
                    Ok(self.event_handler.send_event(wid, wevent))
                }
            },
            MessageCommand::EngineCommand(ec) => match ec {
                EngineCommand::RedrawComplete(wid) => {
                    log::info!("redraw complete message");
                    self.handle_messages();
                    Ok(self
                        .windows
                        .read()
                        .unwrap()
                        .get(&wid)
                        .ok_or(anyhow::anyhow!("window not found"))?
                        .request_redraw())
                }
            },
            _ => Ok(()),
        }
    }

    pub fn set_objects(&mut self, objects: &[SharedBox<dyn Object>]) {
        self.objects = objects.to_vec();
    }
}
