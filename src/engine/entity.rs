use std::{
    any::TypeId,
    collections::HashMap,
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, RwLock},
};

use glam::{Mat4, Vec3};
use three_d::{ColorMaterial, Gm, Mesh};
use uuid::Uuid;
use winit::event::WindowEvent;

use crate::{
    assets::asset_manager::Model,
    utils::{Shared, SharedBox},
};

use super::component::{Component, Transform3D};

#[derive(Clone, Debug)]
pub struct EntityContainer(SharedBox<dyn Entity>);

impl EntityContainer {
    pub fn new(entity: Box<dyn Entity>) -> Self {
        Self(Arc::new(Mutex::new(entity)))
    }

    pub fn id(&self) -> Uuid {
        self.0.lock().as_ref().unwrap().id()
    }
}

impl Deref for EntityContainer {
    type Target = Mutex<Box<dyn Entity>>;
    fn deref(&self) -> &Self::Target {
        &self.0.as_ref()
    }
}

type EntityMap = Arc<RwLock<HashMap<Uuid, EntityContainer>>>;

#[derive(Debug, Clone)]
pub struct EntityRegistry {
    entities: EntityMap,
}

impl EntityRegistry {
    pub fn new() -> Self {
        Self {
            entities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add(&mut self, entity: EntityContainer) {
        self.entities.write().unwrap().insert(entity.id(), entity);
    }

    pub fn remove(&mut self, id: &Uuid) {
        self.entities.write().unwrap().remove(id);
    }

    pub fn get(&self, id: &Uuid) -> Option<EntityContainer> {
        self.entities.read().as_ref().unwrap().get(id).cloned()
    }

    pub fn len(&self) -> usize {
        self.entities.read().as_ref().unwrap().len()
    }
}

impl IntoIterator for EntityRegistry {
    type Item = EntityContainer;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.entities
            .read()
            .unwrap()
            .iter()
            .map(|(_, v)| v.clone())
            .collect::<Vec<EntityContainer>>()
            .into_iter()
    }
}

/// trait for creating game object structs
pub trait Entity: Debug + Send + Sync {
    fn id(&self) -> Uuid;
    fn model(&self) -> &Option<crate::assets::asset_manager::Model>;
    fn transform(&self) -> Transform3D;
    fn transform_mut(&mut self) -> &mut Transform3D;

    fn update(&mut self, delta: f64);
    fn physics_update(&mut self, delta: f64);
    fn input(&mut self, event: &WindowEvent);

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn entity_type(&self) -> TypeId;
    fn clone_box(&self) -> Box<dyn Entity>;

    fn into_container(self) -> EntityContainer;
}

impl Clone for Box<dyn Entity> {
    fn clone(&self) -> Box<dyn Entity> {
        self.clone_box()
    }
}

pub trait Camera: Entity {
    fn view_matrix(&self) -> Mat4;
    fn projection_matrix_lh(&self) -> Mat4;
    fn projection_matrix_rh(&self) -> Mat4;
    fn view_projection_matrix_lh(&self) -> Mat4 {
        self.projection_matrix_lh() * self.view_matrix()
    }
    fn view_projection_matrix_rh(&self) -> Mat4 {
        self.projection_matrix_rh() * self.view_matrix()
    }
}

#[derive(Clone, Debug)]
pub struct DefaultCamera {
    pub transform: Transform3D,
    pub id: Uuid,

    pub width: f32,
    pub height: f32,

    pub up: Vec3,
    pub forward: Vec3,

    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl DefaultCamera {
    pub fn new(
        transform: Transform3D,
        width: f32,
        height: f32,
        up: Vec3,
        forward: Vec3,
        fov: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            transform,
            id: Uuid::new_v4(),
            width,
            height,
            up,
            forward,
            fov,
            near,
            far,
        }
    }
}

impl Entity for DefaultCamera {
    fn id(&self) -> Uuid {
        self.id
    }
    fn model(&self) -> &Option<crate::assets::asset_manager::Model> {
        &None
    }
    fn input(&mut self, event: &WindowEvent) {}
    fn update(&mut self, delta: f64) {}
    fn physics_update(&mut self, delta: f64) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn entity_type(&self) -> TypeId {
        TypeId::of::<DefaultCamera>()
    }
    fn transform(&self) -> Transform3D {
        self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform3D {
        &mut self.transform
    }
    fn clone_box(&self) -> Box<dyn Entity> {
        Box::new(self.clone())
    }
    fn into_container(self) -> EntityContainer {
        EntityContainer::new(Box::new(self))
    }
}

impl Camera for DefaultCamera {
    fn view_matrix(&self) -> Mat4 {
        (self.transform.position_matrix() * self.transform.rotation_matrix()).inverse()
    }

    fn projection_matrix_lh(&self) -> Mat4 {
        let f = 1.0 / f32::tan(self.fov / 2.0);
        let aspect = self.width / self.height;

        #[rustfmt::skip]
        Mat4::from_cols_array(&[
            f / aspect, 0.0, 0.0, 0.0,
            0.0, f, 0.0, 0.0,
            0.0, 0.0, self.far / (self.far - self.near), 1.0,
            0.0, 0.0, (-self.near * self.far) / (self.far - self.near), 0.0,
        ])
    }

    fn projection_matrix_rh(&self) -> Mat4 {
        let f = 1.0 / f32::tan(self.fov / 2.0);
        let aspect = self.width / self.height;

        #[rustfmt::skip]
        Mat4::from_cols_array(&[
            f / aspect, 0.0, 0.0, 0.0,
            0.0, f, 0.0, 0.0,
            0.0, 0.0, (self.far + self.near) / (self.near - self.far), -1.0,
            0.0, 0.0, (2.0 * self.far * self.near) / (self.near - self.far), 0.0,
        ])
    }
}
