use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

use glam::{Mat4, Quat, Vec3};

pub use silly_game_engine_macros::Component;

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

    pub fn has<C: 'static + Component>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<C>())
    }
}

// TODO add tests
#[cfg(test)]
mod component_registry_test {}
