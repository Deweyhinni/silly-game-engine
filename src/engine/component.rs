use std::{
    any::Any,
    fmt::{Debug, Display},
};

use glam::{Quat, Vec3};

pub trait Component: Debug + Send + Sync {
    fn label(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Clone, Copy)]
pub struct Transform3D {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform3D {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }
}

impl Component for Transform3D {
    fn label(&self) -> &str {
        "Transform3D"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
