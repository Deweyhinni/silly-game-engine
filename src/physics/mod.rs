pub mod commands;
pub mod rapier_engine;
use std::{
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};

use crate::{
    engine::{component::Component, entity::EntityRegistry},
    physics::{
        commands::{PhysicsCommand, PhysicsEvent},
        rapier_engine::RapierEngine,
    },
};
use glam::{Quat, Vec3};
use rapier3d::prelude::*;

#[derive(Clone, Debug)]
pub enum RigidBodyState {
    Pending(RigidBody),
    Active(RigidBodyHandle),
    Removed,
}

#[derive(Debug, Clone, Component)]
pub struct PhysicsBody {
    pub collider: Collider,
    pub rigid_body: RigidBodyState,
}

impl PhysicsBody {
    pub fn new(collider: Collider, rigid_body: RigidBody) -> Self {
        Self {
            collider,
            rigid_body: RigidBodyState::Pending(rigid_body),
        }
    }
}

pub struct PhysicsEngine {
    physics_engine: Option<RapierEngine>,
    command_sender: mpsc::Sender<PhysicsCommand>,
    event_receiver: mpsc::Receiver<PhysicsEvent>,

    last_physics_step: Arc<Mutex<Instant>>,
}

impl PhysicsEngine {
    pub fn new(gravity: Vec3, entities: EntityRegistry) -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        let rapier_engine = RapierEngine::new(gravity, entities, command_rx, event_tx);

        Self {
            command_sender: command_tx,
            event_receiver: event_rx,
            physics_engine: Some(rapier_engine),
            last_physics_step: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn start_physics(&mut self) -> anyhow::Result<()> {
        log::debug!("physics started");
        let last_physics_step_mutex = self.last_physics_step.clone();
        let mut rapier_engine = match self.physics_engine.take() {
            Some(pe) => pe,
            None => return Err(anyhow::anyhow!("no physics engine")),
        };
        std::thread::spawn(move || {
            tracy_client::set_thread_name!("Physics Thread");
            loop {
                let _span = tracy_client::span!("physics step");
                let before_step = Instant::now();
                let delta = Instant::now()
                    .duration_since(last_physics_step_mutex.get_cloned().unwrap())
                    .as_millis_f64();
                rapier_engine.step(delta).unwrap();
                let step_time = Instant::now().duration_since(before_step).as_millis_f64();

                std::thread::sleep(Duration::from_millis(
                    10_u64.checked_sub(step_time as u64).unwrap_or(0),
                ));
            }
        });

        Ok(())
    }

    pub fn send_command(&mut self, command: PhysicsCommand) -> anyhow::Result<()> {
        self.command_sender.send(command)?;
        Ok(())
    }
}

#[test]
fn test_component_label() {
    use crate::engine::component::Component;
    let pb = PhysicsBody::new(
        ColliderBuilder::ball(10.0).build(),
        RigidBodyBuilder::dynamic().build(),
    );
    assert_eq!(pb.label(), "PhysicsBody");
}
