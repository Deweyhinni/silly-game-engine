use std::{any::Any, fmt::Debug};

use glam::{Mat4, Quat, Vec3};

/// trait for creating components
pub trait Component: Debug + Send + Sync {
    fn label(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}

/// 3 dimensional transform component
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
    pub fn rotation_matrix(&self) -> Mat4 {
        Mat4::from_quat(self.rotation)
    }
    pub fn position_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position)
    }
    pub fn scale_matrix(&self) -> Mat4 {
        Mat4::from_scale(self.scale)
    }
    pub fn transform_matrix(&self) -> Mat4 {
        self.transform_matrix() * self.rotation_matrix() * self.scale_matrix()
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
