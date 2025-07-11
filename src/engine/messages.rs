use crate::{rendering::RendererCommand, windowing::windower::WindowerCommand};

use super::{EngineCommand, event::EventHandlerCommand};

#[derive(Debug, Clone)]
pub enum Systems {
    Engine,
    EventHandler,
    Renderer,
    Windower,
}

#[derive(Debug, Clone)]
pub enum MessageCommand {
    EngineCommand(EngineCommand),
    RendererCommand(RendererCommand),
    WindowerCommand(WindowerCommand),
    EventHandlerCommand(EventHandlerCommand),
}

#[derive(Debug, Clone)]
pub struct MessageContext {
    pub command: MessageCommand,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub from: Systems,
    pub to: Systems,
    pub context: MessageContext,
}
