use std::collections::HashMap;

use glam::{Quat, Vec3};
use uuid::Uuid;

use crate::engine::systems::System;

#[derive(Debug, Clone, Copy)]
pub struct TransformId(Uuid);

#[derive(Debug, Clone, Copy)]
pub struct BasicTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    parent: TransformId,
    local: BasicTransform,
    global: BasicTransform,
}

/// a registry that stores transforms a manages the hierarchy
#[derive(Debug, Clone)]
pub struct TransformRegistry {
    transforms: HashMap<TransformId, Transform>,
}

impl System for TransformRegistry {
    fn label(&self) -> &str {
        "TransformRegistry"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
