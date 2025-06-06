#![feature(box_into_inner)]

use std::{
    fmt::Display,
    ops::Deref,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use glam::{Mat4, Quat, Vec3};

use game_engine_lib::{
    self,
    engine::{
        component::Transform3D,
        object::{Model, Object},
    },
    rendering::EngineRenderer,
    utils::{Shared, SharedBox, deg_to_rad, new_shared, new_shared_box},
};
use three_d::{ColorMaterial, Context, CpuMaterial, CpuMesh, Gm, Mesh, PhysicalMaterial, Srgba};
use uuid::Uuid;

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

impl Object for TestObj {
    fn id(&self) -> uuid::Uuid {
        Uuid::new_v4()
    }

    fn model(&self) -> Option<SharedBox<dyn game_engine_lib::engine::object::Model>> {
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

    fn clone_box(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }
}

fn main() {
    let mut renderer = EngineRenderer::new(&[]);
    let transform = Transform3D {
        position: Vec3::new(1.0, 0.5, 0.0),
        rotation: Quat::from_axis_angle(
            Vec3::new(1.0, 0.0, 0.0).normalize(),
            deg_to_rad(45.0) as f32,
        ),
        scale: Vec3::new(10.0, 10.0, 10.0),
    };
    let model = TestModel {
        context: renderer.renderer.context.clone(),
    };
    let mut objects: Vec<SharedBox<dyn Object>> = Vec::new();
    objects.push(Arc::new(Mutex::new(Box::new(TestObj::new(
        transform, model,
    )))));
    renderer.set_objects(objects.clone().as_slice());

    thread::spawn(move || {
        let objs = objects.clone();
        for _ in 0..10 {
            thread::sleep(Duration::from_secs(1));
            objs[0]
                .lock()
                .expect("poisoned lock")
                .transform_mut()
                .position
                .x += 1.0;
        }
    });

    renderer.start_render().unwrap();
}
