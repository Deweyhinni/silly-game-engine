use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;

use cgmath::vec3;
use glam::{Mat4, Vec3};
use log::info;
use three_d::{
    Axes, Camera, ClearState, ColorMaterial, Context, CpuMaterial, CpuMesh, CpuTexture,
    DirectionalLight, FlyControl, FrameInput, FrameInputGenerator, FrameOutput, Gm, Mesh, Srgba,
    SurfaceSettings, WindowSettings, WindowedContext, degrees, geometry,
};

use three_d::Object;
use winit::{
    event::WindowEvent,
    window::{Window, WindowId},
};

use crate::engine::entity::{EntityContainer, EntityRegistry};
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
    camera: Camera,
    control: FlyControl,
    lights: Vec<DirectionalLight>,

    objects: EntityRegistry,
    messages: VecDeque<Message>,
}

impl ThreedRenderer {
    /// creates new three_d renderer
    pub fn new(objects: EntityRegistry) -> Self {
        let mut camera = Camera::new_perspective(
            three_d::Viewport::new_at_origo(1, 1),
            vec3(60.0, 50.0, 60.0),
            vec3(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0).into_cgmath(),
            degrees(45.0),
            0.1,
            1000.0,
        );
        let mut control = FlyControl::new(10.);

        let lights = Vec::new();

        Self {
            context: None,
            camera,
            control,
            lights,

            objects,
            messages: VecDeque::new(),
        }
    }

    pub fn init(&mut self, window: &Window) -> anyhow::Result<()> {
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

        Ok(())
    }

    fn render_internal(&mut self, frame_input: &mut FrameInput) -> anyhow::Result<()> {
        let context = self.context.as_ref().ok_or(anyhow::anyhow!("no context"))?;
        let axes = Axes::new(context, 0.5, 10.0);
        self.camera.set_viewport(frame_input.viewport);
        self.control
            .handle_events(&mut self.camera, &mut frame_input.events);

        let delta = frame_input.elapsed_time / 1000.0;

        self.objects.clone().into_iter().for_each(|o| {
            o.lock().expect("poisoned mutex").update(delta);
        });

        let obj_gms: Vec<Vec<_>> = self
            .objects
            .clone()
            .into_iter()
            .filter_map(|o| {
                let transform = o.lock().expect("poisoned mutex").transform();
                let position = transform.position;
                let rotation = transform.rotation;
                let scale = transform.scale;
                let mut gms = match object_get_gm_list(o, &self.context.as_ref().unwrap()) {
                    Ok(gms) => gms,
                    Err(e) => {
                        info!("skipped model because unable to get Gm {e}");
                        return None;
                    }
                };
                let transform_mat = Mat4::from_translation(position)
                    * Mat4::from_quat(rotation)
                    * Mat4::from_scale(scale);
                gms.iter_mut()
                    .for_each(|gm| gm.set_transformation(transform_mat.into_cgmath()));

                Some(gms)
            })
            .collect();

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.5, 0.8, 0.8, 1.0, 1.0))
            .write(|| {
                obj_gms.iter().for_each(|gms| {
                    gms.iter()
                        .for_each(|gm| gm.render(&self.camera, &[&self.lights[0]]))
                });

                axes.render(&self.camera, &[&self.lights[0]]);
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
        // self.init(window);
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

/// takes a reference to an object and gets a list of GM geometry and material instances
fn object_get_gm_list(
    object: EntityContainer,
    context: &WindowedContext,
) -> anyhow::Result<Vec<Gm<Mesh, ColorMaterial>>> {
    let obj = object.clone();
    let model = obj
        .lock()
        .expect("mutex lock failed")
        .model()
        .clone()
        .unwrap();

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
                                    Some(mat) => Some(CpuTexture {
                                        name: "albedo_texture".into(),
                                        data: three_d::TextureData::RgbU8(
                                            mat.albedo
                                                .data
                                                .chunks(3)
                                                .map(|w| [w[0], w[1], w[2]])
                                                .collect(),
                                        ),
                                        width: mat.albedo.width,
                                        height: mat.albedo.height,
                                        min_filter: three_d::Interpolation::Linear,
                                        mag_filter: three_d::Interpolation::Linear,
                                        mipmap: None,
                                        wrap_s: three_d::Wrapping::Repeat,
                                        wrap_t: three_d::Wrapping::Repeat,
                                    }),
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
