#![allow(unused_parens, unused_braces)]

use std::primitive;

use glam::{Vec2, Vec3};

use crate::{
    assets::asset_manager::{self, Material, MeshPrimitive, Model, ModelNode},
    utils::deg_to_rad,
};

pub struct CuboidBuilder {
    hx: f32,
    hy: f32,
    hz: f32,
    color: image::Rgba<u8>,
}

impl CuboidBuilder {
    pub fn new() -> Self {
        Self {
            hx: 1.0,
            hy: 1.0,
            hz: 1.0,
            color: image::Rgba::from([255, 255, 255, 255]),
        }
    }

    pub fn size(mut self, hx: f32, hy: f32, hz: f32) -> Self {
        self.hx = hx;
        self.hy = hy;
        self.hz = hz;
        self
    }

    pub fn color(mut self, color: image::Rgba<u8>) -> Self {
        self.color = color;
        self
    }

    pub fn build(self) -> Model {
        let mesh = asset_manager::Mesh {
            primitives: vec![MeshPrimitive {
                positions: vec![
                    // front face (normal: 0, 0, 1)
                    Vec3::new(-(self.hx / 2.0), -(self.hy / 2.0), (self.hz / 2.0)),
                    Vec3::new((self.hx / 2.0), -(self.hy / 2.0), (self.hz / 2.0)),
                    Vec3::new((self.hx / 2.0), (self.hy / 2.0), (self.hz / 2.0)),
                    Vec3::new(-(self.hx / 2.0), (self.hy / 2.0), (self.hz / 2.0)),
                    // back face (normal: 0, 0, -1)
                    Vec3::new((self.hx / 2.0), -(self.hy / 2.0), -(self.hz / 2.0)),
                    Vec3::new(-(self.hx / 2.0), -(self.hy / 2.0), -(self.hz / 2.0)),
                    Vec3::new(-(self.hx / 2.0), (self.hy / 2.0), -(self.hz / 2.0)),
                    Vec3::new((self.hx / 2.0), (self.hy / 2.0), -(self.hz / 2.0)),
                    // left face (normal: -1, 0, 0)
                    Vec3::new(-(self.hx / 2.0), -(self.hy / 2.0), -(self.hz / 2.0)),
                    Vec3::new(-(self.hx / 2.0), -(self.hy / 2.0), (self.hz / 2.0)),
                    Vec3::new(-(self.hx / 2.0), (self.hy / 2.0), (self.hz / 2.0)),
                    Vec3::new(-(self.hx / 2.0), (self.hy / 2.0), -(self.hz / 2.0)),
                    // right face (normal: 1, 0, 0)
                    Vec3::new((self.hx / 2.0), -(self.hy / 2.0), (self.hz / 2.0)),
                    Vec3::new((self.hx / 2.0), -(self.hy / 2.0), -(self.hz / 2.0)),
                    Vec3::new((self.hx / 2.0), (self.hy / 2.0), -(self.hz / 2.0)),
                    Vec3::new((self.hx / 2.0), (self.hy / 2.0), (self.hz / 2.0)),
                    // top face (normal: 0, 1, 0)
                    Vec3::new(-(self.hx / 2.0), (self.hy / 2.0), (self.hz / 2.0)),
                    Vec3::new((self.hx / 2.0), (self.hy / 2.0), (self.hz / 2.0)),
                    Vec3::new((self.hx / 2.0), (self.hy / 2.0), -(self.hz / 2.0)),
                    Vec3::new(-(self.hx / 2.0), (self.hy / 2.0), -(self.hz / 2.0)),
                    // bottom face (normal: 0, -1, 0)
                    Vec3::new(-(self.hx / 2.0), -(self.hy / 2.0), -(self.hz / 2.0)),
                    Vec3::new((self.hx / 2.0), -(self.hy / 2.0), -(self.hz / 2.0)),
                    Vec3::new((self.hx / 2.0), -(self.hy / 2.0), (self.hz / 2.0)),
                    Vec3::new(-(self.hx / 2.0), -(self.hy / 2.0), (self.hz / 2.0)),
                ],
                normals: vec![
                    // front face
                    Vec3::new(0.0, 0.0, 1.0),
                    Vec3::new(0.0, 0.0, 1.0),
                    Vec3::new(0.0, 0.0, 1.0),
                    Vec3::new(0.0, 0.0, 1.0),
                    // back face
                    Vec3::new(0.0, 0.0, -1.0),
                    Vec3::new(0.0, 0.0, -1.0),
                    Vec3::new(0.0, 0.0, -1.0),
                    Vec3::new(0.0, 0.0, -1.0),
                    // left face
                    Vec3::new(-1.0, 0.0, 0.0),
                    Vec3::new(-1.0, 0.0, 0.0),
                    Vec3::new(-1.0, 0.0, 0.0),
                    Vec3::new(-1.0, 0.0, 0.0),
                    // right face
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(1.0, 0.0, 0.0),
                    // top face
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                    // bottom face
                    Vec3::new(0.0, -1.0, 0.0),
                    Vec3::new(0.0, -1.0, 0.0),
                    Vec3::new(0.0, -1.0, 0.0),
                    Vec3::new(0.0, -1.0, 0.0),
                ],
                tex_coords: vec![
                    // front face
                    Vec2::new(0.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(1.0, 1.0),
                    Vec2::new(0.0, 1.0),
                    // back face
                    Vec2::new(0.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(1.0, 1.0),
                    Vec2::new(0.0, 1.0),
                    // left face
                    Vec2::new(0.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(1.0, 1.0),
                    Vec2::new(0.0, 1.0),
                    // right face
                    Vec2::new(0.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(1.0, 1.0),
                    Vec2::new(0.0, 1.0),
                    // top face
                    Vec2::new(0.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(1.0, 1.0),
                    Vec2::new(0.0, 1.0),
                    // bottom face
                    Vec2::new(0.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(1.0, 1.0),
                    Vec2::new(0.0, 1.0),
                ],
                indices: vec![
                    0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15,
                    12, 16, 17, 18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
                ],
                material_index: None,
            }],
        };

        let model_node = ModelNode {
            transform: glam::Mat4::IDENTITY,
            meshes: vec![mesh],
            nodes: Vec::new(),
        };

        let material = Material {
            albedo: asset_manager::Texture {
                texture_type: asset_manager::TextureType::Albedo,
                image_format: asset_manager::ImageFormat::R8G8B8A8,
                width: 2,
                height: 2,
                data: vec![
                    self.color[0],
                    self.color[1],
                    self.color[2],
                    self.color[3],
                    //
                    self.color[0],
                    self.color[1],
                    self.color[2],
                    self.color[3],
                    //
                    self.color[0],
                    self.color[1],
                    self.color[2],
                    self.color[3],
                    //
                    self.color[0],
                    self.color[1],
                    self.color[2],
                    self.color[3],
                ],
            },
            normals: None,
        };

        Model {
            nodes: vec![model_node],
            materials: vec![material],
        }
    }
}

struct SphereBuilder {
    radius: f32,
    color: image::Rgba<u8>,
    radial_segments: u32,
    rings: u32,
}

impl SphereBuilder {
    pub fn new() -> Self {
        Self {
            radius: 1.0,
            color: image::Rgba::from([255, 255, 255, 255]),
            radial_segments: 64,
            rings: 32,
        }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn color(mut self, color: image::Rgba<u8>) -> Self {
        self.color = color;
        self
    }

    pub fn segments(mut self, radial_segments: u32, rings: u32) -> Self {
        self.radial_segments = radial_segments;
        self.rings = rings;
        self
    }

    pub fn build(self) -> Model {
        todo!()
    }

    fn uv_sphere(&self) -> MeshPrimitive {
        let north_pole = Vec3::new(0.0, self.radius, 0.0);
        let south_pole = Vec3::new(0.0, -self.radius, 0.0);

        let rings: Vec<Vec<_>> = (0..self.rings)
            .map(|r| {
                let ring_y =
                    f32::cos((deg_to_rad(180.0) as f32 / (self.rings - 1) as f32) * (r + 1) as f32);
                let ring: Vec<_> = (0..self.radial_segments)
                    .map(|s| {
                        let rotation =
                            (deg_to_rad(360.0) as f32 / self.radial_segments as f32) * (s as f32);
                        Vec3::new(f32::cos(rotation), ring_y, f32::sin(rotation))
                    })
                    .collect();

                ring
            })
            .collect();

        let indices = {
            let north_pole_indices: Vec<u32> = (1..self.radial_segments)
                .map(|i| vec![0, i, i + 1])
                .flatten()
                .collect();

            let middle_indices: Vec<u32> = (2..self.rings)
                .map(|r| {
                    ((r * self.radial_segments)..=(r * self.radial_segments + self.radial_segments))
                        .map(|i| {
                            vec![
                                i,
                                i - self.radial_segments,
                                i + 1,
                                i + 1,
                                i - self.radial_segments,
                                i + 1 - self.radial_segments,
                            ]
                        })
                        .flatten()
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect();
        };

        todo!()
    }
}
