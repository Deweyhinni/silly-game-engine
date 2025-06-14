use std::{
    clone,
    sync::{Arc, Mutex},
};

use anyhow::anyhow;

use cgmath::vec3;
use glam::{Mat4, Vec3};
use log::info;
use three_d::{
    Axes, Camera, ClearState, ColorMaterial, Context, DirectionalLight, FlyControl, FrameOutput,
    Gm, Mesh, Srgba, Window, WindowSettings, degrees,
};

use three_d::Object as ThreedObject;

use crate::{
    engine::object::{Model, Object},
    utils::{IntoCgmath, SharedBox},
};

use super::Renderer;

/// three_d renderer
pub struct ThreedRenderer {
    window: Window,
    pub context: Context,
    camera: Camera,
    control: FlyControl,
    lights: Vec<DirectionalLight>,

    objects: Vec<SharedBox<dyn Object>>,
}

impl ThreedRenderer {
    /// creates new three_d renderer
    pub fn new(objects: &[SharedBox<dyn Object>]) -> Self {
        let window = Window::new(WindowSettings {
            title: "game engine window".to_string(),
            max_size: Some((1920, 1080)),
            ..Default::default()
        })
        .expect("failed to create window");

        let context = window.gl();

        let mut camera = Camera::new_perspective(
            window.viewport(),
            vec3(60.0, 50.0, 60.0),
            vec3(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0).into_cgmath(),
            degrees(45.0),
            0.1,
            1000.0,
        );
        let mut control = FlyControl::new(10.);

        let lights = Vec::from([DirectionalLight::new(
            &context,
            1.0,
            Srgba::WHITE,
            vec3(0.0, -0.5, -0.5),
        )]);

        Self {
            window,
            context,
            camera,
            control,
            lights,

            objects: objects.to_vec(),
        }
    }
}

impl Renderer for ThreedRenderer {
    /// prepares models for rendering and starts render loop
    fn start_render(mut self) -> anyhow::Result<()> {
        let axes = Axes::new(&self.context, 0.5, 10.0);

        self.window.render_loop(move |mut frame_input| {
            self.camera.set_viewport(frame_input.viewport);
            self.control
                .handle_events(&mut self.camera, &mut frame_input.events);

            let delta = frame_input.elapsed_time / 1000.0;

            self.objects.iter().for_each(|o| {
                o.lock().expect("poisoned mutex").update(delta);
            });

            let obj_gms: Vec<_> = self
                .objects
                .iter()
                .filter_map(|o| {
                    let transform = o.lock().expect("poisoned mutex").transform();
                    let position = transform.position;
                    let rotation = transform.rotation;
                    let scale = transform.scale;
                    let mut gm = match object_get_gm(o) {
                        Ok(gm) => gm,
                        Err(e) => {
                            info!("skipped model because unable to get Gm {e}");
                            return None;
                        }
                    };
                    let transform_mat = Mat4::from_translation(position)
                        * Mat4::from_quat(rotation)
                        * Mat4::from_scale(scale);
                    gm.set_transformation(transform_mat.into_cgmath());

                    Some(gm)
                })
                .collect();

            frame_input
                .screen()
                .clear(ClearState::color_and_depth(0.5, 0.8, 0.8, 1.0, 1.0))
                // .render(&self.camera, &obj_gms, &[&self.lights[0]]);
                .write(|| {
                    for o in obj_gms {
                        o.render(&self.camera, &[&self.lights[0]]);
                    }

                    axes.render(&self.camera, &[&self.lights[0]]);
                    Ok::<(), std::io::Error>(())
                })
                .unwrap();

            FrameOutput::default()
        });

        Ok(())
    }

    fn set_objects(&mut self, objects: &[SharedBox<dyn Object>]) {
        self.objects = objects.to_vec();
    }
}

/// takes a reference to an object and gets a GM geometry and material instance
fn object_get_gm(object: &SharedBox<dyn Object>) -> anyhow::Result<Gm<Mesh, ColorMaterial>> {
    let obj = object.clone();

    Ok(obj
        .lock()
        .expect("poisoned mutex")
        .model()
        .ok_or(anyhow!("missing model"))?
        .lock()
        .expect("poisoned mutex")
        .gm())
}
