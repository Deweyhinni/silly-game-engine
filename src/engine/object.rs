use std::{
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use cgmath::{Quaternion, Vector3};
use three_d::{ColorMaterial, CpuMaterial, Geometry, Gm, Material, Mesh, PhysicalMaterial};
use uuid::Uuid;

/// model trait, made for three_d for now
pub trait Model: Debug + Display {
    fn gm(&self) -> Gm<Mesh, ColorMaterial>;
}

/// 3d transform trait
pub trait Transform: Debug + Display {
    fn position(&self) -> Vector3<f32>;
    fn rotation(&self) -> Quaternion<f32>;
}

/// trait for creating game object structs
pub trait Object: Debug + Display {
    fn id(&self) -> Uuid;
    fn model(&self) -> Option<Arc<Mutex<dyn Model>>>;
    fn transform(&self) -> Arc<Mutex<dyn Transform>>;
}
