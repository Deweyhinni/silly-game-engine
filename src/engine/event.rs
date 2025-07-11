use std::{collections::VecDeque, sync::Weak};

use winit::{event::WindowEvent, window::WindowId};

use super::Engine;

use crate::engine::messages::Message;

#[derive(Debug, Clone)]
pub enum EventHandlerCommand {
    WindowEvent((WindowId, WindowEvent)),
}

pub struct EventHandler {
    pub messages: VecDeque<Message>,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }

    pub fn send_event(&self, window_id: WindowId, event: WindowEvent) -> () {
        todo!("idrk event stuff")
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
