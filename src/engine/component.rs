use std::{
    any::Any,
    fmt::{Debug, Display},
};

use cgmath::{Quaternion, Vector3};

pub trait Component: Debug + Send + Sync {
    fn label(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Clone, Copy)]
pub struct Transform3D {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: f32,
}

impl Transform3D {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>, scale: f32) -> Self {
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
