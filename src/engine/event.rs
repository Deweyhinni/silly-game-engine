use std::{collections::VecDeque, sync::Weak};

use winit::{event::WindowEvent, window::WindowId};

use super::{Engine, entity::EntityRegistry};

use crate::engine::messages::Message;

#[derive(Debug, Clone)]
pub enum EventHandlerCommand {
    WindowEvent((WindowId, WindowEvent)),
}

pub struct EventHandler {
    pub messages: VecDeque<Message>,
    entities: EntityRegistry,
}

impl EventHandler {
    pub fn new(entities: EntityRegistry) -> Self {
        Self {
            messages: VecDeque::new(),
            entities,
        }
    }

    pub fn send_event(&self, window_id: WindowId, event: WindowEvent) -> () {
        // log::debug!("input event: {:?}", event);
        self.entities
            .clone()
            .into_iter()
            .for_each(|e| e.lock().unwrap().input(&event));
    }

    pub fn get_messages(&self) -> &VecDeque<Message> {
        &self.messages
    }

    pub fn get_messages_mut(&mut self) -> &mut VecDeque<Message> {
        &mut self.messages
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }
}
