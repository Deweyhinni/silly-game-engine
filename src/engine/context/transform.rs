use std::collections::HashMap;

use glam::{Quat, Vec3};
use uuid::Uuid;

use crate::engine::component::Component;
use crate::engine::context::{Context, ContextItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransformId(Uuid);

/// a basic transform with translation, rotation, and scale
#[derive(Debug, Clone, Copy, PartialEq)]
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
            translation: (rhs.rotation * self.translation) + rhs.translation,
            rotation: rhs.rotation * self.rotation,
            scale: self.scale + rhs.scale,
        }
    }

    pub fn rotation_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_quat(self.rotation)
    }
    pub fn translation_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_translation(self.translation)
    }
    pub fn scale_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale(self.scale)
    }
    pub fn matrix(&self) -> glam::Mat4 {
        self.translation_matrix() * self.rotation_matrix() * self.scale_matrix()
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

    pub fn local_mut(&mut self) -> &mut BasicTransform {
        &mut self.local
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
    pub fn id(&self) -> TransformId {
        self.id
    }

    /// gets the local transform from the registry if it exists
    pub fn local(&self) -> Option<BasicTransform> {
        let reg = self.context.get::<TransformRegistry>()?;
        Some(reg.read().unwrap().get(self.id)?.local())
    }

    /// gets the global transform from the registry if it exists
    pub fn global(&self) -> Option<BasicTransform> {
        let reg = self.context.get::<TransformRegistry>()?;
        Some(reg.read().unwrap().get(self.id)?.global())
    }

    /// sets the local transform
    pub fn set(&self, transform: BasicTransform) -> Option<()> {
        let reg = self.context.get::<TransformRegistry>()?;
        reg.write().unwrap().set(self.id, transform)
    }

    /// runs provided function on the local basic transform
    pub fn with_mut<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut BasicTransform) -> R,
    {
        let reg = self.context.get::<TransformRegistry>()?;
        let mut reg_t = reg.write().unwrap().get(self.id).unwrap();
        let t_mut = reg_t.local_mut();
        Some(f(t_mut))
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
    ) -> Transform {
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

        Transform {
            id: transform.id,
            context: self.context.clone(),
        }
    }

    pub fn get(&self, id: TransformId) -> Option<RegistryTransform> {
        let mut transform: RegistryTransform = *self.transforms.get(&id)?;
        if let Some(parent) = transform.parent {
            transform.global = self.get(parent)?.global.add(transform.local);
            Some(transform)
        } else {
            transform.global = transform.local;
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

#[cfg(test)]
mod transform_tests {
    use glam::Quat;
    use glam::Vec3;

    use crate::engine::context::transform::BasicTransform;
    use crate::engine::context::transform::TransformRegistry;

    #[test]
    fn test_registry() {
        let mut context = crate::engine::context::Context::new();
        let registry = TransformRegistry::new(context.clone());
        context.add(registry);

        let registry = context.get::<TransformRegistry>().unwrap();

        let t1: super::Transform = registry.write().unwrap().transform(
            Vec3::new(1.0, 1.0, 1.0),
            Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            None,
        );
        let t2: super::Transform = registry.write().unwrap().transform(
            Vec3::new(1.0, 1.0, 1.0),
            Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Some(t1.id()),
        );

        assert_eq!(
            t1.local().unwrap().add(t2.local().unwrap()),
            t2.global().unwrap()
        );

        t1.set(BasicTransform::new(
            Vec3::new(2.0, 2.0, 2.0),
            Quat::from_euler(glam::EulerRot::XYZ, 1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
        ));

        assert_eq!(
            t1.local().unwrap().add(t2.local().unwrap()),
            t2.global().unwrap()
        );
    }

    #[test]
    fn test_set() {
        let mut context = crate::engine::context::Context::new();
        let registry = TransformRegistry::new(context.clone());
        context.add(registry);

        let registry = context.get::<TransformRegistry>().unwrap();

        let t1: super::Transform = registry.write().unwrap().transform(
            Vec3::new(1.0, 1.0, 1.0),
            Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            None,
        );

        let new_transform = BasicTransform {
            translation: Vec3::new(10.0, 10.0, 10.0),
            rotation: Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        };

        t1.set(new_transform).unwrap();

        assert_eq!(t1.local().unwrap(), new_transform);
    }
}
