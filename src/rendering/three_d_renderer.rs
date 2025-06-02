use std::{
    clone,
    sync::{Arc, Mutex},
};

use anyhow::anyhow;

use cgmath::{Matrix4, vec3};
use log::info;
use three_d::{
    Axes, Camera, ClearState, ColorMaterial, Context, CpuMaterial, CpuMesh, CpuModel,
    DirectionalLight, FlyControl, FrameOutput, Gm, Mat4, Mesh, PhysicalMaterial, Srgba, Vec3,
    Window, WindowSettings, degrees,
};

use crate::engine::object::{Model, Object};

use super::Renderer;

/// three_d renderer
pub struct ThreedRenderer {
    window: Window,
    pub context: Context,
    camera: Camera,
    control: FlyControl,
    lights: Vec<DirectionalLight>,

    objects: Vec<Box<dyn Object>>,
}

impl ThreedRenderer {
    /// creates new three_d renderer
    pub fn new(objects: &[Box<dyn Object>]) -> Self {
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
            Vec3::unit_y(),
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
        let obj_gms: Vec<_> = self
            .objects
            .iter()
            .filter_map(|o| {
                let transform = o.clone().transform().clone();
                let position = transform.position();
                let rotation = transform.rotation();
                let scale = transform.scale();
                let mut gm = match object_get_gm(o) {
                    Ok(gm) => gm,
                    Err(e) => {
                        info!("skipped model because enable to get Gm {e}");
                        return None;
                    }
                };
                let transform_mat = Matrix4::from_translation(position)
                    * Matrix4::from(rotation)
                    * Matrix4::from_scale(scale);
                gm.set_transformation(transform_mat);

                Some(gm)
            })
            .collect();

        let axes = Axes::new(&self.context, 0.5, 10.0);

        self.window.render_loop(move |mut frame_input| {
            self.camera.set_viewport(frame_input.viewport);
            self.control
                .handle_events(&mut self.camera, &mut frame_input.events);

            frame_input
                .screen()
                .clear(ClearState::color_and_depth(0.5, 0.8, 0.8, 1.0, 1.0))
                .render(&self.camera, &obj_gms, &[&self.lights[0]]);

            FrameOutput::default()
        });

        Ok(())
    }

    fn set_objects(&mut self, objects: &[Box<dyn Object>]) {
        self.objects = objects.to_vec();
    }
}

/// takes a reference to an object and gets a GM geometry and material instance
fn object_get_gm(object: &Box<dyn Object>) -> anyhow::Result<Gm<Mesh, ColorMaterial>> {
    let obj = object.clone();

    Ok(obj.model().ok_or(anyhow!("missing model"))?.gm())
}
