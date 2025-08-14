#![allow(unused)]
#![feature(box_into_inner)]

use std::{
    any::TypeId,
    fmt::Display,
    ops::Deref,
    path::Path,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Duration,
};

use glam::{Mat4, Quat, Vec3};

use game_engine_lib::{
    self,
    assets::asset_manager::{Asset, AssetManager, Model},
    engine::{
        Engine,
        component::Transform3D,
        entity::{DefaultCamera, Entity, EntityContainer, EntityRegistry},
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
    event::WindowEvent,
    keyboard::Key,
    window::WindowAttributes,
};

#[derive(Debug, Clone)]
pub struct TestObj {
    transform: Transform3D,
    model: Option<Model>,
    id: Uuid,
}

impl TestObj {
    pub fn new(transform: Transform3D, model: Option<Model>) -> Self {
        Self {
            transform,
            model,
            id: Uuid::new_v4(),
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
        self.id
    }

    fn model(&self) -> &Option<Model> {
        &self.model
    }

    fn transform(&self) -> Transform3D {
        self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform3D {
        &mut self.transform
    }

    fn update(&mut self, delta: f64) {
        // self.transform.position.x += 1.0 * delta as f32;
        self.transform.rotation =
            self.transform.rotation * Quat::from_rotation_y(deg_to_rad(200.0 * delta) as f32);
    }

    fn physics_update(&mut self, delta: f64) {
        ()
    }

    fn input(&mut self, event: &winit::event::WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                log::debug!("{:?}", event.logical_key)
            }
            e => log::debug!("event: {:?}", e),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn entity_type(&self) -> std::any::TypeId {
        TypeId::of::<TestObj>()
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

    let mut asset_manager = AssetManager::new();

    let transform = Transform3D {
        position: Vec3::new(0.0, 0.5, 0.0),
        rotation: Quat::from_axis_angle(
            Vec3::new(1.0, 0.0, 0.0).normalize(),
            deg_to_rad(0.0) as f32,
        ),
        scale: Vec3::new(10.0, 10.0, 10.0),
    };

    let (uuid, maybe_model) = asset_manager
        .get_asset_by_path(Path::new("DamagedHelmet.glb"))
        .expect("model not found");
    let model = match maybe_model.as_ref() {
        Asset::Model(m) => m,
        _ => panic!("model isnt model"),
    };

    let camera = DefaultCamera::new(
        Transform3D {
            position: Vec3::new(0.0, 100.0, 0.0),
            rotation: Quat::from_euler(glam::EulerRot::XYZ, 180.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        1920.0,
        1080.0,
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, -1.0),
        deg_to_rad(90.0) as f32,
        0.1,
        500.0,
    );

    let camera_id = camera.id();

    entities.add(TestObj::new(transform, Some(model.clone())).into_container());
    entities.add(camera.into_container());

    let mut engine = Engine::new(RendererType::ThreeD, entities.clone(), camera_id);

    let mut windower = Windower::new(
        engine,
        WindowAttributes::default()
            .with_title("meow")
            .with_position(LogicalPosition::new(0, 0))
            .with_inner_size(LogicalSize::new(1280, 720)),
    );

    windower.run().unwrap();
}
