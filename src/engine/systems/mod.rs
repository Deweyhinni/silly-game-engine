use std::{
    any::Any,
    any::TypeId,
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

/// trait for systems
pub trait System: Debug {
    fn label(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// type alias for storing systems
pub type SystemMap = Arc<RwLock<HashMap<TypeId, Box<dyn System>>>>;

/// Registry that stores and controls systems
#[derive(Clone, Debug)]
pub struct SystemRegistry {
    systems: SystemMap,
}
