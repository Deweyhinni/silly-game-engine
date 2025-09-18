use std::primitive;

use glam::{Vec2, Vec3};

use crate::assets::asset_manager::{self, Material, MeshPrimitive, Model, ModelNode};

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
