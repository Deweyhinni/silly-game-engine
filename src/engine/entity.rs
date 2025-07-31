use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, RwLock},
};

use three_d::{ColorMaterial, Gm, Mesh};
use uuid::Uuid;
use winit::event::WindowEvent;

use crate::{
    assets::asset_manager::Model,
    utils::{Shared, SharedBox},
};

use super::component::Transform3D;

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

pub type EntityMap = Arc<RwLock<HashMap<Uuid, EntityContainer>>>;

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

/// model trait, made for three_d for now
// pub trait Model: Debug + Display + Send + Sync {
//     fn model(&self) -> crate::assets::asset_manager::Model;
//
//     fn as_any(&self) -> &dyn std::any::Any;
//     fn clone_box(&self) -> Box<dyn Model>;
// }

/// trait for creating game object structs
pub trait Entity: Debug + Display + Send + Sync {
    fn id(&self) -> Uuid;
    fn model(&self) -> &Option<crate::assets::asset_manager::Model>;
    fn transform(&self) -> Transform3D;
    fn transform_mut(&mut self) -> &mut Transform3D;

    fn update(&mut self, delta: f64);
    fn physics_update(&mut self, delta: f64);
    fn input(&mut self, event: &WindowEvent);

    fn as_any(&self) -> &dyn std::any::Any;
    fn clone_box(&self) -> Box<dyn Entity>;

    fn into_container(self) -> EntityContainer;
}

impl Clone for Box<dyn Entity> {
    fn clone(&self) -> Box<dyn Entity> {
        self.clone_box()
    }
}
