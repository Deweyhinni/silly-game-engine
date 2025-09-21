use glam::{Quat, Vec3};
use uuid::Uuid;

pub enum PhysicsCommand {
    Enable {
        id: Uuid,
    },
    Disable {
        id: Uuid,
    },
    ApplyForce {
        id: Uuid,
        force: Vec3,
    },
    ApplyTorque {
        id: Uuid,
        torque: Vec3,
    },
    ApplyImpulse {
        id: Uuid,
        impulse: Vec3,
    },
    ApplyTorqueImpulse {
        id: Uuid,
        impulse: Vec3,
    },
    SetLinearVelocity {
        id: Uuid,
        velocity: Vec3,
    },
    SetAngularVelocity {
        id: Uuid,
        velocity: Vec3,
    },
    SetPosition {
        id: Uuid,
        translation: Vec3,
        rotation: Quat,
    },
    SetTranslation {
        id: Uuid,
        translation: Vec3,
    },
    SetRotation {
        id: Uuid,
        translation: Vec3,
    },
    SetDensity {
        id: Uuid,
        density: f32,
    },
}

pub enum PhysicsEvent {}
