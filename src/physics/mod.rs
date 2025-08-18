pub mod rapier_engine;
use crate::engine::component::{Component, ComponentDerive};
use rapier3d::prelude::*;

#[derive(Debug, Clone, ComponentDerive)]
pub struct PhysicsBody {
    pub collider: Collider,
    pub rigid_body: RigidBody,
}

impl PhysicsBody {
    pub fn new(collider: Collider, rigid_body: RigidBody) -> Self {
        Self {
            collider,
            rigid_body,
        }
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
