use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

use glam::{Mat4, Quat, Vec3};

pub use silly_game_engine_macros::Component as ComponentDerive;

/// trait for creating components
pub trait Component: Debug + Send + Sync {
    fn label(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn Component>;
}

impl Clone for Box<dyn Component> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct ComponentSet {
    components: HashMap<TypeId, Box<dyn Component>>,
}

impl ComponentSet {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn add<C: 'static + Component>(&mut self, component: C) {
        self.components
            .insert(TypeId::of::<C>(), Box::new(component));
    }

    pub fn remove<C: 'static + Component>(&mut self) -> Option<Box<dyn Component>> {
        self.components.remove(&TypeId::of::<C>())
    }

    pub fn get<C: 'static + Component>(&self) -> Option<&C> {
        self.components
            .get(&TypeId::of::<C>())
            .and_then(|boxed| boxed.as_any().downcast_ref::<C>())
    }

    pub fn get_mut<C: 'static + Component>(&mut self) -> Option<&mut C> {
        self.components
            .get_mut(&TypeId::of::<C>())
            .and_then(|boxed| boxed.as_any_mut().downcast_mut::<C>())
    }

    pub fn has<C: 'static + Component>(&mut self) -> bool {
        self.components.contains_key(&TypeId::of::<C>())
    }
}

#[cfg(test)]
mod component_registry_test {
    use super::{ComponentSet, Transform3D};

    #[test]
    fn add_get_eq() {
        let mut cr = ComponentSet::new();
        let transform_c = Transform3D::new(
            glam::Vec3::new(1.0, 1.0, 1.0),
            glam::Quat::from_euler(glam::EulerRot::XYZ, 1.0, 0.0, 0.0),
            glam::Vec3::new(1.0, 1.0, 1.0),
        );
        cr.add(transform_c.clone());
        let transform_c_2 = cr.get::<Transform3D>().unwrap();
        assert_eq!(&transform_c, transform_c_2);
    }
}

/// 3 dimensional transform component
#[derive(Debug, Clone, Copy, PartialEq, ComponentDerive)]
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
        self.position_matrix() * self.rotation_matrix() * self.scale_matrix()
    }
}
