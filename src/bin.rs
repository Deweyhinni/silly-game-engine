#![allow(unused)]
#![feature(box_into_inner)]

use std::{
    fmt::Display,
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Duration,
};

use glam::{Mat4, Quat, Vec3};

use game_engine_lib::{
    self,
    engine::{
        Engine,
        component::Transform3D,
        entity::{Entity, EntityContainer, EntityRegistry, Model},
        event::EventHandler,
    },
    rendering::{EngineRenderer, RendererType},
    utils::{Shared, SharedBox, deg_to_rad, new_shared, new_shared_box},
    windowing::windower::Windower,
};
use three_d::{ColorMaterial, Context, CpuMaterial, CpuMesh, Gm, Mesh, PhysicalMaterial, Srgba};
use uuid::Uuid;
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    window::WindowAttributes,
};

#[derive(Debug, Clone)]
pub struct TestModel {
    context: Context,
}

impl TestModel {
    pub fn new(context: &Context) -> Self {
        Self {
            context: context.clone(),
        }
    }
}

impl Display for TestModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Model for TestModel {
    fn gm(&self) -> three_d::Gm<three_d::Mesh, three_d::ColorMaterial> {
        Gm::new(
            Mesh::new(&self.context, &CpuMesh::cube()),
            ColorMaterial {
                color: Srgba::RED,
                ..Default::default()
            },
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Model> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct TestObj {
    transform: Transform3D,
    model: SharedBox<TestModel>,
}

impl TestObj {
    pub fn new(transform: Transform3D, model: TestModel) -> Self {
        Self {
            transform,
            model: new_shared_box(model),
        }
    }
}

impl Display for TestObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Entity for TestObj {
    fn id(&self) -> uuid::Uuid {
        Uuid::new_v4()
    }

    fn model(&self) -> Option<SharedBox<dyn game_engine_lib::engine::entity::Model>> {
        let model_clone = Box::into_inner(self.model.lock().expect("poisoned mutex").clone());
        Some(Arc::new(Mutex::new(Box::new(model_clone))))
    }

    fn transform(&self) -> Transform3D {
        self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform3D {
        &mut self.transform
    }

    fn update(&mut self, delta: f64) {
        self.transform.position.x += 1.0 * delta as f32;
    }

    fn physics_update(&mut self, delta: f64) {
        ()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Entity> {
        Box::new(self.clone())
    }

    fn into_container(self) -> EntityContainer {
        EntityContainer::new(Box::new(self))
    }
}

fn main() {
    env_logger::init();
    log::info!("logger init");
    let mut entities = EntityRegistry::new();
    let mut engine = Engine::new(RendererType::ThreeD, entities.clone());

    // let transform = Transform3D {
    //     position: Vec3::new(1.0, 0.5, 0.0),
    //     rotation: Quat::from_axis_angle(
    //         Vec3::new(1.0, 0.0, 0.0).normalize(),
    //         deg_to_rad(45.0) as f32,
    //     ),
    //     scale: Vec3::new(10.0, 10.0, 10.0),
    // };
    //
    // let context = engine.renderer.renderer.context.as_ref().unwrap().deref();
    //
    // let model = TestModel {
    //     context: context.clone(),
    // };
    //
    // entities.add(TestObj::new(transform, model).into_container());

    let mut windower = Windower::new(
        engine,
        WindowAttributes::default()
            .with_title("meow")
            .with_position(LogicalPosition::new(0, 0))
            .with_inner_size(LogicalSize::new(1280, 720)),
    );

    windower.run().unwrap();
}
