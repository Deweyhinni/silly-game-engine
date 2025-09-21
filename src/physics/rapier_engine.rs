use std::sync::mpsc::{Receiver, Sender};

use glam::{Quat, Vec3};
use rapier3d::prelude::*;
use uuid::Uuid;

use crate::{
    engine::entity::EntityRegistry,
    physics::{
        PhysicsBody, RigidBodyState,
        commands::{PhysicsCommand, PhysicsEvent},
    },
};

pub struct RapierEngine {
    pub gravity: Vec3,

    command_receiver: Receiver<PhysicsCommand>,
    event_sender: Sender<PhysicsEvent>,

    entities: EntityRegistry,

    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,

    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
}

impl RapierEngine {
    pub fn new(
        gravity: Vec3,
        entities: EntityRegistry,
        command_receiver: Receiver<PhysicsCommand>,
        event_sender: Sender<PhysicsEvent>,
    ) -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        for e in entities.clone().into_iter() {
            let transform = e.lock().unwrap().transform();
            let mut entity = e.lock().unwrap();
            let body: &mut PhysicsBody = match entity.components_mut().get_mut::<PhysicsBody>() {
                Some(pb) => pb,
                None => {
                    continue;
                }
            };
            let rigid_body = match &mut body.rigid_body {
                RigidBodyState::Pending(rb) => rb,
                RigidBodyState::Active(_) => {
                    log::debug!(
                        "Weird: entity body skipped in rapier engine creation because rigid body is already active"
                    );
                    continue;
                }
                RigidBodyState::Removed => {
                    log::debug!(
                        "Weird: entity body skipped in rapier engine creation because it has been removed"
                    );
                    continue;
                }
            };

            rigid_body.set_position((transform.position, transform.rotation).into(), true);

            let rb_handle = rigid_body_set.insert(rigid_body.clone());
            body.rigid_body = RigidBodyState::Active(rb_handle);
            collider_set.insert_with_parent(body.collider.clone(), rb_handle, &mut rigid_body_set);
        }

        Self {
            gravity,
            command_receiver,
            event_sender,
            entities,
            rigid_body_set,
            collider_set,
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    pub fn step(&mut self, delta: f64) -> anyhow::Result<()> {
        let physics_hooks = ();
        let event_handler = ();

        let commands: Vec<PhysicsCommand> = self.command_receiver.try_iter().collect();

        for pc in commands {
            match self.handle_command(pc) {
                Ok(()) => (),
                Err(e) => {
                    log::debug!("skipped physics command: {}", e);
                }
            }
        }

        self.physics_pipeline.step(
            &self.gravity.into(),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &physics_hooks,
            &event_handler,
        );

        for e in self.entities.clone().into_iter() {
            let mut entity = e.lock().unwrap();
            let pb = match entity.components().get::<PhysicsBody>() {
                Some(pb) => pb,
                None => continue,
            };
            let rb = match &pb.rigid_body {
                RigidBodyState::Active(handle) => self.rigid_body_set.get(*handle).unwrap(),
                RigidBodyState::Pending(rb) => rb,
                RigidBodyState::Removed => {
                    log::debug!("skipped update for removed rigid body");
                    continue;
                }
            };

            let rb_pos = *rb.position();

            entity.transform_mut().position = Vec3 {
                x: rb_pos.translation.x,
                y: rb_pos.translation.y,
                z: rb_pos.translation.z,
            };

            entity.transform_mut().rotation = Quat::from(rb_pos.rotation);
        }

        Ok(())
    }

    fn handle_command(&mut self, command: PhysicsCommand) -> anyhow::Result<()> {
        match command {
            PhysicsCommand::ApplyForce { id, force } => self.apply_force(id, force),
            _ => Err(anyhow::anyhow!(
                "i haven't done this physics command yet lol"
            )),
        }
    }

    fn apply_force(&mut self, id: Uuid, force: Vec3) -> anyhow::Result<()> {
        match self.entities.get(&id) {
            Some(e) => match e.lock().unwrap().components().get::<PhysicsBody>() {
                Some(pb) => match &pb.rigid_body {
                    RigidBodyState::Active(handle) => {
                        match self.rigid_body_set.get_mut(handle.clone()) {
                            Some(rb) => Ok(rb.add_force(force.into(), true)),
                            None => {
                                Err(anyhow::anyhow!("rigid body handle leads to no rigid body"))
                            }
                        }
                    }
                    RigidBodyState::Removed => Err(anyhow::anyhow!("rigid body has been removed")),
                    RigidBodyState::Pending(_rb) => {
                        Err(anyhow::anyhow!("cannot mutate pending body"))
                    }
                },
                None => Err(anyhow::anyhow!("entity has no physics body component")),
            },
            None => Err(anyhow::anyhow!("no entity with provided id found")),
        }
    }
}
