use std::collections::HashMap;

use glam::{Quat, Vec3};
use uuid::Uuid;

use crate::engine::component::Component;
use crate::engine::context::{Context, ContextItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransformId(Uuid);

/// a basic transform with translation, rotation, and scale
#[derive(Debug, Clone, Copy)]
pub struct BasicTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl BasicTransform {
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn add(self, rhs: Self) -> Self {
        Self {
            translation: self.translation + rhs.translation,
            rotation: rhs.rotation * self.rotation,
            scale: self.scale + rhs.scale,
        }
    }
}

/// the transform struct that gets stored in the registry
#[derive(Debug, Clone, Copy)]
pub struct RegistryTransform {
    id: TransformId,
    parent: Option<TransformId>,
    local: BasicTransform,
    global: BasicTransform,
}

impl RegistryTransform {
    pub fn id(&self) -> TransformId {
        self.id
    }
    pub fn local(&self) -> BasicTransform {
        self.local
    }

    pub fn global(&self) -> BasicTransform {
        self.global
    }
}

/// a transform handle component
#[derive(Clone, Debug, Component)]
pub struct Transform {
    id: TransformId,
    context: Context,
}

impl Transform {
    pub fn local(&self) -> Option<BasicTransform> {
        let reg = self.context.get::<TransformRegistry>()?;
        Some(reg.read().unwrap().get(self.id)?.local())
    }

    pub fn global(&self) -> Option<BasicTransform> {
        let reg = self.context.get::<TransformRegistry>()?;
        Some(reg.read().unwrap().get(self.id)?.global())
    }

    pub fn set(&self, transform: BasicTransform) -> Option<()> {
        let reg = self.context.get::<TransformRegistry>()?;
        reg.write().unwrap().set(self.id, transform)
    }
}

/// a registry that stores transforms a manages the hierarchy
#[derive(Debug, Clone)]
pub struct TransformRegistry {
    transforms: HashMap<TransformId, RegistryTransform>,
    context: Context,
}

impl TransformRegistry {
    pub fn new(context: Context) -> Self {
        Self {
            transforms: HashMap::new(),
            context,
        }
    }

    pub fn transform(
        &mut self,
        translation: Vec3,
        rotation: Quat,
        scale: Vec3,
        parent: Option<TransformId>,
    ) -> TransformId {
        let transform = RegistryTransform {
            id: TransformId(Uuid::new_v4()),
            parent,
            local: BasicTransform {
                translation,
                rotation,
                scale,
            },
            global: BasicTransform {
                translation,
                rotation,
                scale,
            },
        };

        self.transforms.insert(transform.id, transform);

        transform.id
    }

    pub fn get(&self, id: TransformId) -> Option<RegistryTransform> {
        let mut transform: RegistryTransform = *self.transforms.get(&id)?;
        if let Some(parent) = transform.parent {
            transform.global = self.get(parent)?.global.add(transform.local);
            Some(transform)
        } else {
            Some(transform)
        }
    }

    pub fn set(&mut self, id: TransformId, transform: BasicTransform) -> Option<()> {
        Some(self.transforms.get_mut(&id)?.local = transform)
    }
}

impl ContextItem for TransformRegistry {
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
