use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use anyhow::anyhow;

use cgmath::vec3;
use glam::{Mat4, Vec3};
use log::info;
use three_d::{
    Axes, Camera, ClearState, ColorMaterial, Context, CpuMaterial, CpuMesh, CpuTexture,
    DirectionalLight, FlyControl, FrameInput, FrameInputGenerator, FrameOutput, Gm, Mesh, Srgba,
    SurfaceSettings, TextureData, WindowSettings, WindowedContext, degrees, geometry, radians,
};

use three_d::Object;
use uuid::Uuid;
use winit::{
    event::WindowEvent,
    window::{Window, WindowId},
};

use crate::engine::context::transform::{BasicTransform, Transform};
use crate::engine::entity::{DefaultCamera, EntityContainer, EntityRegistry};
use crate::engine::messages::Message;
use crate::{
    assets::asset_manager::Model,
    engine::{Engine, entity::Entity},
    utils::{IntoCgmath, SharedBox, WeakShared},
};

use super::Renderer;

/// three_d renderer
pub struct ThreedRenderer {
    // window_id: WindowId,
    pub context: Option<WindowedContext>,
    camera: Option<Camera>,
    camera_id: Option<Uuid>,
    control: FlyControl,
    lights: Vec<DirectionalLight>,

    objects: EntityRegistry,
    object_gm_cache: HashMap<Uuid, Vec<Gm<Mesh, ColorMaterial>>>,
    messages: VecDeque<Message>,
}

impl ThreedRenderer {
    /// creates new three_d renderer
    pub fn new(objects: EntityRegistry) -> Self {
        let mut control = FlyControl::new(10.);

        let lights = Vec::new();

        Self {
            context: None,
            camera: None,
            camera_id: None,
            control,
            lights,

            objects,
            object_gm_cache: HashMap::new(),
            messages: VecDeque::new(),
        }
    }

    pub fn init(&mut self, window: &Window, camera_id: &Uuid) -> anyhow::Result<()> {
        let camera_container = self
            .objects
            .get(camera_id)
            .ok_or(anyhow::anyhow!("camera not found from provided id"))?;

        let camera_lock = camera_container.lock().expect("mutex lock failed");

        let camera_entity = camera_lock
            .as_any()
            .downcast_ref::<DefaultCamera>()
            .ok_or(anyhow::anyhow!("provided entity is not a camera"))?;

        let mut camera = {
            let cam_transform = camera_entity
                .components()
                .get::<Transform>()
                .ok_or(anyhow::anyhow!("no transform component on camere"))?;
            let global_transform = cam_transform
                .global()
                .ok_or(anyhow::anyhow!("unable to get transform from registry"))?;
            let pos = global_transform.translation;
            let rotation = global_transform.rotation;
            let target = Vec3::from(pos + rotation * camera_entity.forward);

            Camera::new_perspective(
                three_d::Viewport::new_at_origo(1, 1),
                pos.into_cgmath(),
                target.into_cgmath(),
                camera_entity.up.into_cgmath(),
                radians(camera_entity.fov),
                camera_entity.near,
                camera_entity.far,
            )
        };

        let context =
            WindowedContext::from_winit_window(window, SurfaceSettings::default()).unwrap();

        let lights = [DirectionalLight::new(
            &context,
            1.0,
            Srgba::WHITE,
            vec3(0.0, -0.5, -0.5),
        )];

        self.context = Some(context);
        self.lights = Vec::from(lights);
        self.camera = Some(camera);
        self.camera_id = Some(*camera_id);

        Ok(())
    }

    fn render_internal(&mut self, frame_input: &mut FrameInput) -> anyhow::Result<()> {
        let context = self.context.as_ref().ok_or(anyhow::anyhow!("no context"))?;
        let axes = Axes::new(context, 0.5, 10.0);

        let camera_container = self
            .objects
            .get(
                &self
                    .camera_id
                    .ok_or(anyhow::anyhow!("no camera id"))
                    .unwrap(),
            )
            .ok_or(anyhow::anyhow!("no camera entity"))
            .unwrap();

        let cam_global_t = {
            let cam_lock = camera_container.lock().expect("mutex lock failed");
            let cam_components = cam_lock.components();

            let cam_transform = cam_components
                .get::<Transform>()
                .ok_or(anyhow::anyhow!("no transform component on camera"))?;

            cam_transform
                .global()
                .ok_or(anyhow::anyhow!("unable to get transform from registry"))?
        };

        let pos = cam_global_t.translation;
        let rotation = cam_global_t.rotation;
        let target = Vec3::from(pos + rotation * Vec3::new(0.0, 0.0, -1.0));

        self.camera
            .as_mut()
            .ok_or(anyhow::anyhow!("no camera"))?
            .set_view(
                pos.into_cgmath(),
                target.into_cgmath(),
                Vec3::new(0.0, 1.0, 0.0).into_cgmath(),
            );

        self.camera
            .as_mut()
            .ok_or(anyhow::anyhow!("no camera"))?
            .set_viewport(frame_input.viewport);

        let delta = frame_input.elapsed_time;

        self.objects.clone().into_iter().for_each(|o| {
            o.lock().expect("poisoned mutex").update(delta);
        });

        self.objects.clone().into_iter().for_each(|o| {
            let global_transform = {
                let o_lock = o.lock().expect("poisoned mutex");
                let o_components = o_lock.components();
                let o_transform = match o_components.get::<Transform>() {
                    Some(t) => t,
                    None => {
                        log::info!("skipped object render because it has no transform component");
                        return;
                    }
                };

                let global_transform = match o_transform.global() {
                    Some(t) => t,
                    None => {
                        log::info!("skipped object render: unable to get transform from registry");
                        return;
                    }
                };

                global_transform
            };

            if !self.object_gm_cache.contains_key(&o.id()) {
                let mut gms = match object_get_gm_list(o.clone(), &self.context.as_ref().unwrap()) {
                    Ok(g) => g,
                    Err(e) => {
                        log::info!("skipped object render because unable to get gm list: {e}");
                        return;
                    }
                };
                gms.iter_mut()
                    .for_each(|gm| gm_update_transform(gm, &global_transform));
                self.object_gm_cache.insert(o.id(), gms);
            };

            if let Some(gms) = self.object_gm_cache.get_mut(&o.id()) {
                gms.iter_mut()
                    .for_each(|gm| gm_update_transform(gm, &global_transform));
            };
        });

        let objs_gms: Vec<&Vec<_>> = self
            .objects
            .clone()
            .into_iter()
            .filter_map(|o| {
                let gms = match self.object_gm_cache.get(&o.id()) {
                    Some(g) => g,
                    None => return None,
                };

                Some(gms)
            })
            .collect();

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.5, 0.8, 0.8, 1.0, 1.0))
            .write(|| {
                objs_gms.iter().for_each(|gms| {
                    gms.iter().for_each(|gm| {
                        gm.render(&self.camera.as_ref().unwrap(), &[&self.lights[0]])
                    })
                });

                axes.render(&self.camera.as_ref().unwrap(), &[&self.lights[0]]);
                Ok::<(), std::io::Error>(())
            })
            .unwrap();

        context.swap_buffers().unwrap();

        Ok(())
    }
}

impl Renderer for ThreedRenderer {
    /// prepares models for rendering and starts render loop
    fn render(&mut self, window: Arc<Window>) -> anyhow::Result<()> {
        let mut frame_input_generator = FrameInputGenerator::from_winit_window(window.as_ref());
        let context = self
            .context
            .as_ref()
            .ok_or(anyhow::anyhow!("no render context"))?;

        context.make_current().unwrap();

        self.render_internal(&mut frame_input_generator.generate(context))?;
        window.request_redraw();
        Ok(())
    }

    fn handle_resize(&mut self, window: Arc<Window>, event: &WindowEvent) -> anyhow::Result<()> {
        match event {
            WindowEvent::Resized(physical_size) => {
                self.context
                    .as_ref()
                    .ok_or(anyhow::anyhow!("no render context"))?
                    .resize(*physical_size);
            }
            _ => return Err(anyhow::anyhow!("not the correct event")),
        }

        Ok(())
    }

    fn handle_scale_factor_change(
        &mut self,
        window: Arc<Window>,
        event: &WindowEvent,
    ) -> anyhow::Result<()> {
        match event {
            winit::event::WindowEvent::ScaleFactorChanged {
                inner_size_writer, ..
            } => {
                todo!()
            }

            _ => return Err(anyhow::anyhow!("not the correct event")),
        }

        Ok(())
    }

    fn handle_close(&mut self, window: Arc<Window>, event: &WindowEvent) -> anyhow::Result<()> {
        match event {
            WindowEvent::CloseRequested => {
                self.context
                    .as_ref()
                    .ok_or(anyhow::anyhow!("no render context"))?
                    .make_current()
                    .unwrap();
            }
            _ => return Err(anyhow::anyhow!("not the correct event")),
        }

        Ok(())
    }

    fn set_objects(&mut self, objects: EntityRegistry) {
        self.objects = objects;
    }

    fn get_messages(&self) -> &VecDeque<Message> {
        &self.messages
    }

    fn get_messages_mut(&mut self) -> &mut VecDeque<Message> {
        &mut self.messages
    }

    fn clear_messages(&mut self) {
        self.messages.clear();
    }
}

fn gm_update_transform(gm: &mut Gm<Mesh, ColorMaterial>, transform: &BasicTransform) {
    let transform_mat = Mat4::from_translation(transform.translation)
        * Mat4::from_quat(transform.rotation)
        * Mat4::from_scale(transform.scale);
    gm.set_transformation(transform_mat.into_cgmath());
}

/// takes a reference to an object and gets a list of GM geometry and material instances
fn object_get_gm_list(
    object: EntityContainer,
    context: &WindowedContext,
) -> anyhow::Result<Vec<Gm<Mesh, ColorMaterial>>> {
    let _span = tracy_client::span!("getting geometry and material from entity");
    let obj = object.clone();
    let model = obj
        .lock()
        .expect("mutex lock failed")
        .model()
        .clone()
        .ok_or(anyhow::anyhow!("no model in entity"))?;

    let node_list = model.get_nodes_flattened();
    let gms = node_list
        .iter()
        .map(|node| {
            node.meshes
                .iter()
                .map(|mesh| {
                    mesh.primitives
                        .iter()
                        .map(|prim| {
                            let geometry = mesh_prim_to_geometry(prim, context)
                                .ok_or(anyhow::anyhow!("unable to create geometry from primitive"))
                                .unwrap();

                            let cpu_texture = match prim.material_index {
                                Some(index) => match model.materials.get(index) {
                                    Some(mat) => {
                                        let albedo_data = match mat.albedo.image_format {
                                            crate::assets::asset_manager::ImageFormat::R8G8B8 => {
                                                TextureData::RgbU8(mat.albedo.data.chunks(3).map(|c| [c[0], c[1], c[2]]).collect())
                                            }
                                            crate::assets::asset_manager::ImageFormat::R8G8B8A8 => {
                                                TextureData::RgbaU8(mat.albedo.data.chunks(4).map(|c| [c[0], c[1], c[2], c[3]]).collect())

                                            }
                                        };
                                        Some(CpuTexture {
                                        name: "albedo_texture".into(),
                                        data: albedo_data,
                                        width: mat.albedo.width,
                                        height: mat.albedo.height,
                                        min_filter: three_d::Interpolation::Linear,
                                        mag_filter: three_d::Interpolation::Linear,
                                        mipmap: None,
                                        wrap_s: three_d::Wrapping::Repeat,
                                        wrap_t: three_d::Wrapping::Repeat,
                                    })},
                                    None => None,
                                },
                                None => None,
                            };

                            let material = three_d::ColorMaterial::new(
                                context,
                                &CpuMaterial {
                                    albedo: Srgba {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        a: 255,
                                    },
                                    albedo_texture: cpu_texture,
                                    ..Default::default()
                                },
                            );

                            Gm::new(geometry, material)
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>();

    Ok(gms)
}

fn mesh_prim_to_geometry(
    prim: &crate::assets::asset_manager::MeshPrimitive,
    context: &WindowedContext,
) -> Option<three_d::Mesh> {
    let cpu_mesh = CpuMesh {
        positions: three_d::Positions::F32(
            prim.positions.iter().map(|p| p.into_cgmath()).collect(),
        ),
        indices: three_d::Indices::U32(prim.indices.clone()),
        normals: Some(prim.normals.iter().map(|n| n.into_cgmath()).collect()),
        uvs: Some(prim.tex_coords.iter().map(|tc| tc.into_cgmath()).collect()),
        tangents: None,
        colors: None,
    };

    Some(three_d::Mesh::new(context, &cpu_mesh))
}
