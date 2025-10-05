use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

pub mod transform;

/// a trait for context items
pub trait ContextItem: Debug + Send + Sync + 'static {
    fn label(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

struct ContextEntry<T: ContextItem> {
    inner: Arc<RwLock<T>>,
}

impl<T: ContextItem> ContextEntry<T> {
    fn new(item: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(item)),
        }
    }

    fn get(&self) -> Arc<RwLock<T>> {
        Arc::clone(&self.inner)
    }
}

type AnyMap = HashMap<TypeId, Box<dyn Any + Send + Sync>>;

/// a context registry that holds global context needed for running a world
#[derive(Debug, Clone)]
pub struct Context {
    items: Arc<RwLock<AnyMap>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add<C: 'static + ContextItem>(&mut self, item: C) {
        let entry = ContextEntry::new(item);
        self.items
            .write()
            .unwrap()
            .insert(TypeId::of::<C>(), Box::new(entry));
    }

    pub fn get<C: 'static + ContextItem>(&self) -> Option<Arc<RwLock<C>>> {
        let items = self.items.read().unwrap();
        let entry = items.get(&TypeId::of::<C>())?;
        let entry = entry.downcast_ref::<ContextEntry<C>>()?;
        Some(entry.get())
    }
}
