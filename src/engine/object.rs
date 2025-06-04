use std::{
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use three_d::{ColorMaterial, CpuMaterial, Geometry, Gm, Material, Mesh, PhysicalMaterial};
use uuid::Uuid;

use crate::utils::{Shared, SharedBox};

use super::component::Transform3D;

/// model trait, made for three_d for now
pub trait Model: Debug + Display + Send + Sync {
    fn gm(&self) -> Gm<Mesh, ColorMaterial>;

    fn as_any(&self) -> &dyn std::any::Any;
    fn clone_box(&self) -> Box<dyn Model>;
}

/// trait for creating game object structs
pub trait Object: Debug + Display + Send + Sync {
    fn id(&self) -> Uuid;
    fn model(&self) -> Option<SharedBox<dyn Model>>;
    fn transform(&self) -> Transform3D;
    fn transform_mut(&mut self) -> &mut Transform3D;

    fn update(&mut self, delta: f64);
    fn physics_update(&mut self, delta: f64);

    fn as_any(&self) -> &dyn std::any::Any;
    fn clone_box(&self) -> Box<dyn Object>;
}

impl Clone for Box<dyn Object> {
    fn clone(&self) -> Box<dyn Object> {
        self.clone_box()
    }
}

impl Clone for Box<dyn Model> {
    fn clone(&self) -> Box<dyn Model> {
        self.clone_box()
    }
}
