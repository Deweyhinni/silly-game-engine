use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use cgmath::{Deg, Quaternion, Rotation3, Vector3};
use game_engine_lib::{
    self,
    engine::object::{Model, Object, Transform},
    rendering::EngineRenderer,
    utils::{SharedBox, new_shared_box},
};
use three_d::{ColorMaterial, Context, CpuMaterial, CpuMesh, Gm, Mesh, PhysicalMaterial, Srgba};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub struct TestTransform {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: f32,
}

impl Display for TestTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}; {:?}", self.position, self.rotation)
    }
}

impl Transform for TestTransform {
    fn position(&self) -> Vector3<f32> {
        self.position
    }
    fn rotation(&self) -> Quaternion<f32> {
        self.rotation
    }
    fn scale(&self) -> f32 {
        self.scale
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn clone_box(&self) -> Box<dyn Transform> {
        Box::new(self.clone())
    }
}

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
            // PhysicalMaterial::new_transparent(
            //     &self.context,
            //     &CpuMaterial {
            //         albedo: Srgba {
            //             r: 255,
            //             g: 0,
            //             b: 0,
            //             a: 220,
            //         },
            //         ..Default::default()
            //     },
            // ),
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
    transform: Box<TestTransform>,
    model: Box<TestModel>,
}

impl TestObj {
    pub fn new(transform: TestTransform, model: TestModel) -> Self {
        Self {
            transform: Box::new(transform),
            model: Box::new(model),
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

    fn model(&self) -> Option<Box<dyn game_engine_lib::engine::object::Model>> {
        Some(self.model.clone())
    }

    fn transform(&self) -> Box<dyn Transform> {
        self.transform.clone()
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
    let transform = TestTransform {
        position: Vector3::new(0.0, 0.0, 0.0),
        rotation: Quaternion::from_axis_angle(Vector3::new(1.0, 0.0, 0.0), Deg(45.0)),
        scale: 10.0,
    };
    let model = TestModel {
        context: renderer.renderer.context.clone(),
    };
    let mut objects: Vec<SharedBox<dyn Object>> = Vec::new();
    objects.push(Arc::new(Mutex::new(Box::new(TestObj::new(
        transform, model,
    )))));
    renderer.set_objects(objects.as_slice());

    renderer.start_render().unwrap();
}
