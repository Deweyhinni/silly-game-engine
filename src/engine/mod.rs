use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock, atomic::AtomicU64},
    time::{Duration, Instant},
};

use entity::{Entity, EntityRegistry};
use event::{EventHandler, EventHandlerCommand};
use messages::{Message, MessageCommand};
use uuid::Uuid;
use winit::window::{Window, WindowId};

use crate::{
    physics::{PhysicsEngine, rapier_engine::RapierEngine},
    rendering::{EngineRenderer, Renderer, RendererCommand, RendererType},
};

pub mod component;
pub mod entity;
pub mod event;
pub mod messages;
pub mod systems;

#[derive(Debug, Clone)]
pub enum EngineCommand {
    RedrawComplete(WindowId),
}

pub struct Engine {
    pub renderer: EngineRenderer,
    pub event_handler: EventHandler,
    pub physics_engine: PhysicsEngine,

    windows: Arc<RwLock<HashMap<WindowId, Arc<Window>>>>,
    pub default_camera_id: Uuid,
    pub objects: EntityRegistry,

    last_frame_render: Instant,
}

impl Engine {
    pub fn new(
        renderer_type: RendererType,
        entities: EntityRegistry,
        default_camera_id: Uuid,
    ) -> Self {
        Self {
            renderer: EngineRenderer::new(renderer_type, entities.clone()),
            event_handler: EventHandler::new(entities.clone()),
            physics_engine: PhysicsEngine::new(
                glam::Vec3 {
                    x: 0.0,
                    y: -9.81,
                    z: 0.0,
                },
                entities.clone(),
            ),
            windows: Arc::new(RwLock::new(HashMap::new())),
            default_camera_id,
            objects: entities,
            last_frame_render: Instant::now(),
        }
    }

    pub fn init(
        &mut self,
        windows: &Arc<RwLock<HashMap<WindowId, Arc<Window>>>>,
    ) -> anyhow::Result<()> {
        self.handle_messages();

        self.windows = Arc::clone(&windows);

        self.last_frame_render = Instant::now();

        self.start_physics().unwrap();

        Ok(())
    }

    pub fn start_physics(&mut self) -> anyhow::Result<()> {
        self.physics_engine.start_physics()
    }

    /// handles the rendering of a frame
    pub fn handle_render(&mut self, window: Arc<Window>) {
        let delta = Instant::now()
            .duration_since(self.last_frame_render)
            .as_millis_f64();
        self.last_frame_render = Instant::now();

        // self.rapier_engine.step(delta).unwrap();

        self.renderer.render(window).unwrap();
    }

    pub fn handle_messages(&mut self) {
        let mut msg_queues = [
            self.event_handler.get_messages().clone(),
            self.renderer.get_messages().clone(),
            self.objects
                .clone()
                .into_iter()
                .map(|e| {
                    let msgs = e.lock().unwrap().get_messages().clone();
                    e.lock().unwrap().clear_messages();
                    msgs
                })
                .flatten()
                .collect(),
        ];

        self.event_handler.clear_messages();
        self.renderer.clear_messages();

        log::info!("messages: {:?}", msg_queues);

        for queue in msg_queues.iter_mut() {
            while !queue.is_empty() {
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

    pub fn handle_message(&mut self, msg: Message) -> anyhow::Result<()> {
        match msg.context.command {
            MessageCommand::RendererCommand(rc) => match rc {
                RendererCommand::Render(wid) => self.renderer.render(Arc::clone(
                    self.windows
                        .read()
                        .unwrap()
                        .get(&wid)
                        .ok_or(anyhow::anyhow!("window not found"))?,
                )),
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
            MessageCommand::PhysicsCommand(phc) => self.physics_engine.send_command(phc),
            _ => Ok(()),
        }
    }

    pub fn set_objects(&mut self, objects: EntityRegistry) {
        self.objects = objects;
    }
}
