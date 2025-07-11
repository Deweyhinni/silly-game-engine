use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;

use cgmath::vec3;
use glam::{Mat4, Vec3};
use log::info;
use three_d::{
    Axes, Camera, ClearState, ColorMaterial, Context, DirectionalLight, FlyControl, FrameInput,
    FrameInputGenerator, FrameOutput, Gm, Mesh, Srgba, SurfaceSettings, WindowSettings,
    WindowedContext, degrees,
};

use three_d::Object as ThreedObject;
use winit::{
    event::WindowEvent,
    window::{Window, WindowId},
};

use crate::engine::messages::Message;
use crate::{
    engine::{
        Engine,
        object::{Model, Object},
    },
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

    objects: Vec<SharedBox<dyn Object>>,
    messages: VecDeque<Message>,
}

impl ThreedRenderer {
    /// creates new three_d renderer
    pub fn new(objects: &[SharedBox<dyn Object>]) -> Self {
        log::info!("meow 2");

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

            objects: objects.to_vec(),
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
            .write(|| {
                for o in obj_gms {
                    o.render(&self.camera, &[&self.lights[0]]);
                }

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
        log::info!("rendering");
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

    fn set_objects(&mut self, objects: &[SharedBox<dyn Object>]) {
        self.objects = objects.to_vec();
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
