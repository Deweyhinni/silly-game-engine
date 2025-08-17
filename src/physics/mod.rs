pub mod rapier_engine;
use crate::engine::component::{Component, ComponentDerive};

#[derive(Debug, Clone, ComponentDerive)]
pub struct PhysicsBody {}

#[test]
fn test_component_label() {
    use crate::engine::component::Component;
    let pb = PhysicsBody {};
    assert_eq!(pb.label(), "PhysicsBody");
}
