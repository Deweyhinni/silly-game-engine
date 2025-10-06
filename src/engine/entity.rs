use std::{
    any::{Any, TypeId},
    collections::{HashMap, VecDeque},
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
    engine::{
        component::ComponentSet,
        context::{
            Context,
            transform::{BasicTransform, Transform, TransformRegistry},
        },
        messages::Message,
    },
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
    context: Context,
}

impl EntityRegistry {
    pub fn new(context: Context) -> Self {
        Self {
            entities: Arc::new(RwLock::new(HashMap::new())),
            context,
        }
    }

    pub fn add(&mut self, entity: EntityContainer) {
        entity.lock().unwrap().set_context(self.context.clone());
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

    fn set_context(&mut self, context: Context);

    fn update(&mut self, delta: f64);
    fn physics_update(&mut self, delta: f64);
    fn input(&mut self, event: &WindowEvent);

    fn components(&self) -> &ComponentSet;
    fn components_mut(&mut self) -> &mut ComponentSet;

    fn get_messages(&self) -> &VecDeque<Message>;
    fn clear_messages(&mut self);

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

#[derive(Debug, Clone, Component)]
pub struct Children {
    parent: Uuid,
    children: Vec<Uuid>,
    entities: EntityRegistry,
}

impl Children {
    pub fn new(parent: Uuid, children: Vec<EntityContainer>, entities: EntityRegistry) -> Self {
        let mut entities = entities;
        let children_ids = children
            .iter()
            .map(|c| {
                let c_id = c.id();
                c.lock()
                    .unwrap()
                    .components_mut()
                    .add(Parent::new(parent, c_id));
                entities.add(c.clone());
                c_id
            })
            .collect();
        Self {
            parent,
            children: children_ids,
            entities,
        }
    }

    pub fn get(&self) -> Vec<EntityContainer> {
        self.children
            .iter()
            .filter_map(|id| self.entities.get(id))
            .collect()
    }

    pub fn get_ids(&self) -> &[Uuid] {
        &self.children
    }

    pub fn get_by_type<E: 'static + Entity>(&self) -> Vec<EntityContainer> {
        self.children
            .iter()
            .filter_map(|c_id| {
                self.entities.get(c_id).and_then(|e| {
                    if e.lock().unwrap().entity_type() == TypeId::of::<E>() {
                        Some(e)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Component)]
pub struct Parent {
    parent: Uuid,
    child: Uuid,
}

impl Parent {
    pub fn new(parent: Uuid, child: Uuid) -> Self {
        Self { parent, child }
    }

    pub fn get_id(&self) -> Uuid {
        self.parent
    }
}

/// camera trait
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
    components: ComponentSet,
    pub id: Uuid,
    messages: VecDeque<Message>,
    context: Context,

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
        transform: BasicTransform,
        context: Context,
        width: f32,
        height: f32,
        up: Vec3,
        forward: Vec3,
        fov: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let mut components = ComponentSet::new();
        let transform = context
            .get::<TransformRegistry>()
            .expect("no transform registry in context")
            .write()
            .unwrap()
            .transform(
                transform.translation,
                transform.rotation,
                transform.scale,
                None,
            );
        components.add(transform);

        Self {
            components,
            id: Uuid::new_v4(),
            messages: VecDeque::new(),
            context,
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
    fn set_context(&mut self, context: Context) {
        self.context = context;
    }
    fn components(&self) -> &ComponentSet {
        &self.components
    }
    fn components_mut(&mut self) -> &mut ComponentSet {
        &mut self.components
    }
    fn get_messages(&self) -> &VecDeque<Message> {
        &self.messages
    }
    fn clear_messages(&mut self) {
        self.messages.clear();
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
        (self
            .components
            .get::<Transform3D>()
            .unwrap()
            .position_matrix()
            * self
                .components
                .get::<Transform3D>()
                .unwrap()
                .rotation_matrix())
        .inverse()
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
