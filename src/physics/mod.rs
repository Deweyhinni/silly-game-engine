pub mod rapier_engine;
use crate::engine::component::{Component, ComponentDerive};
use glam::{Quat, Vec3};
use rapier3d::prelude::*;

#[derive(Clone, Debug)]
pub enum RigidBodyState {
    Pending(RigidBody),
    Active(RigidBodyHandle),
    Removed,
}

#[derive(Debug, Clone, ComponentDerive)]
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

#[test]
fn test_component_label() {
    use crate::engine::component::Component;
    let pb = PhysicsBody::new(
        ColliderBuilder::ball(10.0).build(),
        RigidBodyState::Pending(RigidBodyBuilder::dynamic().build()),
    );
    assert_eq!(pb.label(), "PhysicsBody");
}
