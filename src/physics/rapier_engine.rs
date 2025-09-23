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
            let _span = tracy_client::span!("modifying entities");
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
        let _span = tracy_client::span!("handling command");
        match command {
            PhysicsCommand::ApplyForce { id, force } => self.apply_force(id, force),
            PhysicsCommand::ApplyTorque { id, torque } => self.apply_torque(id, torque),
            PhysicsCommand::ApplyImpulse { id, impulse } => self.apply_impulse(id, impulse),
            PhysicsCommand::ApplyTorqueImpulse { id, impulse } => {
                self.apply_torque_impulse(id, impulse)
            }
            PhysicsCommand::SetLinearVelocity { id, velocity } => {
                self.set_linear_velocity(id, velocity)
            }
            PhysicsCommand::SetAngularVelocity { id, velocity } => {
                self.set_angular_velocity(id, velocity)
            }
            PhysicsCommand::SetPosition {
                id,
                translation,
                rotation,
            } => self.set_position(id, translation, rotation),
            PhysicsCommand::SetTranslation { id, translation } => {
                self.set_translation(id, translation)
            }
            PhysicsCommand::SetRotation { id, rotation } => self.set_rotation(id, rotation),

            _ => Err(anyhow::anyhow!(
                "i haven't done this physics command yet lol"
            )),
        }
    }

    fn apply_force(&mut self, id: Uuid, force: Vec3) -> anyhow::Result<()> {
        self.run_on_rb(id, |rb| {
            rb.add_force(force.into(), true);
        })
    }

    fn apply_torque(&mut self, id: Uuid, torque: Vec3) -> anyhow::Result<()> {
        self.run_on_rb(id, |rb| {
            rb.add_torque(torque.into(), true);
        })
    }

    fn apply_impulse(&mut self, id: Uuid, impulse: Vec3) -> anyhow::Result<()> {
        self.run_on_rb(id, |rb| {
            rb.apply_impulse(impulse.into(), true);
        })
    }

    fn apply_torque_impulse(&mut self, id: Uuid, impulse: Vec3) -> anyhow::Result<()> {
        self.run_on_rb(id, |rb| {
            rb.apply_torque_impulse(impulse.into(), true);
        })
    }

    fn set_linear_velocity(&mut self, id: Uuid, velocity: Vec3) -> anyhow::Result<()> {
        self.run_on_rb(id, |rb| {
            rb.set_linvel(velocity.into(), true);
        })
    }

    fn set_angular_velocity(&mut self, id: Uuid, velocity: Vec3) -> anyhow::Result<()> {
        self.run_on_rb(id, |rb| {
            rb.set_angvel(velocity.into(), true);
        })
    }

    fn set_position(&mut self, id: Uuid, translation: Vec3, rotation: Quat) -> anyhow::Result<()> {
        self.run_on_rb(id, |rb| {
            rb.set_position((translation, rotation).into(), true);
        })
    }

    fn set_translation(&mut self, id: Uuid, translation: Vec3) -> anyhow::Result<()> {
        self.run_on_rb(id, |rb| {
            rb.set_translation(translation.into(), true);
        })
    }

    fn set_rotation(&mut self, id: Uuid, rotation: Quat) -> anyhow::Result<()> {
        self.run_on_rb(id, |rb| {
            rb.set_rotation(rotation.into(), true);
        })
    }

    fn run_on_rb<F>(&mut self, id: Uuid, mut op: F) -> anyhow::Result<()>
    where
        F: FnMut(&mut RigidBody),
    {
        match self.entities.get(&id) {
            Some(e) => match e.lock().unwrap().components().get::<PhysicsBody>() {
                Some(pb) => match &pb.rigid_body {
                    RigidBodyState::Active(handle) => {
                        match self.rigid_body_set.get_mut(handle.clone()) {
                            Some(rb) => Ok(op(rb)),
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
