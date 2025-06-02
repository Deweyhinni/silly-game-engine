use std::{
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use cgmath::{Quaternion, Vector3};
use three_d::{ColorMaterial, CpuMaterial, Geometry, Gm, Material, Mesh, PhysicalMaterial};
use uuid::Uuid;

/// model trait, made for three_d for now
pub trait Model: Debug + Display + Send + Sync {
    fn gm(&self) -> Gm<Mesh, ColorMaterial>;

    fn as_any(&self) -> &dyn std::any::Any;
    fn clone_box(&self) -> Box<dyn Model>;
}

/// 3d transform trait
pub trait Transform: Debug + Display + Send + Sync {
    fn position(&self) -> Vector3<f32>;
    fn rotation(&self) -> Quaternion<f32>;
    fn scale(&self) -> f32;

    fn as_any(&self) -> &dyn std::any::Any;
    fn clone_box(&self) -> Box<dyn Transform>;
}

/// trait for creating game object structs
pub trait Object: Debug + Display + Send + Sync {
    fn id(&self) -> Uuid;
    fn model(&self) -> Option<Box<dyn Model>>;
    fn transform(&self) -> Box<dyn Transform>;

    fn as_any(&self) -> &dyn std::any::Any;
    fn clone_box(&self) -> Box<dyn Object>;
}

impl Clone for Box<dyn Object> {
    fn clone(&self) -> Box<dyn Object> {
        self.clone_box()
    }
}

impl Clone for Box<dyn Transform> {
    fn clone(&self) -> Box<dyn Transform> {
        self.clone_box()
    }
}

impl Clone for Box<dyn Model> {
    fn clone(&self) -> Box<dyn Model> {
        self.clone_box()
    }
}
