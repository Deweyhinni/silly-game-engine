use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use cgmath::{Deg, Quaternion, Rotation3, Vector3};
use game_engine_lib::{
    self,
    engine::object::{Model, Object, Transform},
    rendering::EngineRenderer,
};
use three_d::{ColorMaterial, Context, CpuMaterial, CpuMesh, Gm, Mesh, PhysicalMaterial, Srgba};
use uuid::Uuid;

#[derive(Debug)]
pub struct TestTransform {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
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
}

#[derive(Debug)]
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
}

#[derive(Debug)]
pub struct TestObj {
    transform: Arc<Mutex<TestTransform>>,
    model: Arc<Mutex<TestModel>>,
}

impl TestObj {
    pub fn new(transform: TestTransform, model: TestModel) -> Self {
        Self {
            transform: Arc::new(Mutex::new(transform)),
            model: Arc::new(Mutex::new(model)),
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

    fn model(&self) -> Option<Arc<Mutex<dyn game_engine_lib::engine::object::Model>>> {
        Some(self.model.clone())
    }

    fn transform(&self) -> Arc<Mutex<dyn Transform>> {
        self.transform.clone()
    }
}

fn main() {
    let mut renderer = EngineRenderer::new(&[]);
    let transform = TestTransform {
        position: Vector3::new(0.0, 0.0, 0.0),
        rotation: Quaternion::from_axis_angle(Vector3::new(1.0, 0.0, 1.0), Deg(25.0)),
    };
    let model = TestModel {
        context: renderer.renderer.context.clone(),
    };
    let mut objects: Vec<Arc<Mutex<dyn Object>>> = Vec::new();
    objects.push(Arc::new(Mutex::new(TestObj::new(transform, model))));
    renderer.set_objects(objects.as_slice());

    renderer.start_render().unwrap();
}
