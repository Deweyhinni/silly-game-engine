#![allow(unused)]
#![feature(box_into_inner)]

use std::{
    any::TypeId,
    collections::VecDeque,
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
    assets::{
        asset_manager::{Asset, AssetManager, Model},
        basic_models,
    },
    engine::{
        Engine,
        component::{ComponentRegistry, Transform3D},
        entity::{DefaultCamera, Entity, EntityContainer, EntityRegistry},
        event::EventHandler,
        messages::Message,
    },
    physics::{PhysicsBody, commands::PhysicsCommand},
    rendering::{EngineRenderer, RendererType},
    utils::{Shared, SharedBox, deg_to_rad, new_shared, new_shared_box},
    windowing::windower::Windower,
};
use rapier3d::prelude::{ColliderBuilder, RigidBodyBuilder};
use three_d::{ColorMaterial, Context, CpuMaterial, CpuMesh, Gm, Mesh, PhysicalMaterial, Srgba};
use uuid::Uuid;
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{Key, KeyCode, PhysicalKey},
    window::WindowAttributes,
};

use silly_game_engine_macros;

#[derive(Debug, Clone)]
pub struct TestObj {
    transform: Transform3D,
    model: Option<Model>,
    components: ComponentRegistry,
    messages: VecDeque<Message>,
    id: Uuid,
}

impl TestObj {
    pub fn new(
        transform: Transform3D,
        model: Option<Model>,
        components: ComponentRegistry,
    ) -> Self {
        Self {
            transform,
            model,
            id: Uuid::new_v4(),
            messages: VecDeque::new(),
            components,
        }
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id
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
        // self.transform.rotation =
        //     self.transform.rotation * Quat::from_rotation_y(deg_to_rad(200.0 * delta) as f32);

        self.messages.push_back(Message {
            from: game_engine_lib::engine::messages::Systems::Engine,
            to: game_engine_lib::engine::messages::Systems::Physics,
            context: game_engine_lib::engine::messages::MessageContext {
                command: game_engine_lib::engine::messages::MessageCommand::PhysicsCommand(
                    PhysicsCommand::ApplyForce {
                        id: self.id,
                        force: Vec3::new(0.0, 0.0, 1.0) * delta as f32,
                    },
                ),
            },
        });
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
                match event {
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state: ElementState::Pressed,
                        ..
                    } => match keycode {
                        KeyCode::KeyW => {
                            self.transform.position.z += 1.0;
                        }
                        KeyCode::KeyS => {
                            self.transform.position.z -= 1.0;
                        }
                        KeyCode::KeyA => {
                            self.transform.position.x += 1.0;
                        }
                        KeyCode::KeyD => {
                            self.transform.position.x -= 1.0;
                        }
                        KeyCode::Space => {
                            self.transform.position.y += 1.0;
                        }
                        KeyCode::ShiftLeft => {
                            self.transform.position.y -= 1.0;
                        }
                        KeyCode::ArrowLeft => {
                            self.transform.rotation = self.transform.rotation
                                * Quat::from_euler(
                                    glam::EulerRot::XYZ,
                                    0.0,
                                    deg_to_rad(10.0) as f32,
                                    0.0,
                                )
                        }
                        _ => (),
                    },
                    _ => (),
                }
                log::debug!("{:?}", event.logical_key)
            }
            e => log::debug!("event: {:?}", e),
        }
    }

    fn components(&self) -> &ComponentRegistry {
        &self.components
    }
    fn components_mut(&mut self) -> &mut ComponentRegistry {
        &mut self.components
    }

    fn get_messages(
        &self,
    ) -> &std::collections::VecDeque<game_engine_lib::engine::messages::Message> {
        &self.messages
    }
    fn clear_messages(&mut self) {
        self.messages.clear();
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
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
    tracy_client::Client::start();

    let mut entities = EntityRegistry::new();

    let mut asset_manager = AssetManager::new();

    let transform = Transform3D {
        position: Vec3::new(0.0, 300.0, 0.0),
        rotation: Quat::from_axis_angle(
            Vec3::new(1.0, 0.0, 0.0).normalize(),
            deg_to_rad(0.0) as f32,
        ),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };

    let lantern_model = {
        let (uuid, maybe_model) = asset_manager
            .get_asset_by_path(Path::new("Lantern.glb"))
            .expect("model not found");
        match maybe_model.as_ref() {
            Asset::Model(m) => m.clone(),
            _ => panic!("model isn't model"),
        }
    };

    let avocado_model = {
        let (uuid, maybe_model) = asset_manager
            .get_asset_by_path(Path::new("DamagedHelmet.glb"))
            .expect("model not found");
        match maybe_model.as_ref() {
            Asset::Model(m) => m.clone(),
            _ => panic!("model isn't model"),
        }
    };

    let camera = DefaultCamera::new(
        Transform3D {
            position: Vec3::new(50.0, 75.0, -50.0),
            rotation: Quat::from_euler(
                glam::EulerRot::XYZ,
                deg_to_rad(210.0) as f32,
                deg_to_rad(30.0) as f32,
                0.0,
            ),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        1920.0,
        1080.0,
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, -1.0),
        deg_to_rad(120.0) as f32,
        0.1,
        500.0,
    );

    let camera_id = camera.id();

    let mut components = ComponentRegistry::new();
    let pb = PhysicsBody::new(
        ColliderBuilder::cuboid(5.0, 20.0, 5.0).build(),
        RigidBodyBuilder::dynamic().build(),
    );
    components.add(pb);

    let test_obj = TestObj::new(transform, Some(lantern_model), components);

    let plane = TestObj::new(
        Transform3D {
            position: Vec3::new(0.0, -0.5, 0.0),
            rotation: Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        Some(
            basic_models::CuboidBuilder::new()
                .size(100.0, 20.0, 100.0)
                .build(),
        ),
        {
            let mut creg = ComponentRegistry::new();
            creg.add(PhysicsBody::new(
                ColliderBuilder::cuboid(100.0, 20.0, 100.0).build(),
                RigidBodyBuilder::fixed().build(),
            ));

            creg
        },
    );

    let avocado = TestObj::new(
        Transform3D {
            position: Vec3::new(0.0, 100.0, 1.0),
            rotation: Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            scale: Vec3::new(10.0, 10.0, 10.0),
        },
        Some(avocado_model),
        {
            let mut creg = ComponentRegistry::new();
            creg.add(PhysicsBody::new(
                ColliderBuilder::ball(1.0).build(),
                RigidBodyBuilder::dynamic().build(),
            ));
            creg
        },
    );

    entities.add(camera.into_container());
    entities.add(plane.into_container());
    entities.add(test_obj.into_container());
    entities.add(avocado.into_container());

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
