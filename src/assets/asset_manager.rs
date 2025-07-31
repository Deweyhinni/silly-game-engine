use std::{
    collections::HashMap,
    hash, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use glam::{Mat4, Vec2, Vec3};
use gltf::{Document, Scene};

use include_dir;
use include_dir::Dir;
use uuid::Uuid;

static ASSET_DIR: Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/assets");

#[derive(Clone, Debug)]
pub struct MeshPrimitive {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub tex_coords: Vec<Vec2>,
    pub indices: Vec<u32>,
    pub material_index: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub primitives: Vec<MeshPrimitive>,
}

#[derive(Clone, Debug, Copy)]
pub enum TextureType {
    Albedo,
    Normal,
    Roughness,
}

#[derive(Clone, Debug, Copy)]
pub enum ImageFormat {
    R8G8B8,
    R8G8B8A8,
}

#[derive(Clone, Debug)]
pub struct Texture {
    pub texture_type: TextureType,
    pub image_format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Material {
    pub albedo: Texture,
    pub normals: Option<Texture>,
}

#[derive(Clone, Debug)]
pub struct ModelNode {
    pub transform: Mat4,
    pub meshes: Vec<Mesh>,
    pub nodes: Vec<ModelNode>,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub nodes: Vec<ModelNode>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn get_nodes_flattened(&self) -> Vec<ModelNode> {
        Self::get_nodes_recurse(&self.nodes, Mat4::IDENTITY)
    }

    fn get_nodes_recurse(nodes: &Vec<ModelNode>, upper_transform: Mat4) -> Vec<ModelNode> {
        nodes
            .iter()
            .map(|n| {
                let node = ModelNode {
                    transform: upper_transform * n.transform,
                    meshes: n.meshes.clone(),
                    nodes: Vec::new(),
                };
                let mut children = Model::get_nodes_recurse(&n.nodes, node.transform);
                children.insert(0, node);
                children
            })
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_simple_hierarchy() {
        let child_node = ModelNode {
            transform: Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            meshes: vec![],
            nodes: vec![],
        };

        let root_node = ModelNode {
            transform: Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            meshes: vec![],
            nodes: vec![child_node],
        };

        let model = Model {
            nodes: vec![root_node],
            materials: vec![],
        };

        let flattened = model.get_nodes_flattened();

        // We should get 2 nodes: the root and the child with combined transform
        assert_eq!(flattened.len(), 2);

        // First node should be the root with its original transform
        let first_node = &flattened[0];
        assert_eq!(
            first_node.transform,
            Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0))
        );
        assert!(first_node.nodes.is_empty());

        // Second node should be the child with combined transform (root * child)
        let second_node = &flattened[1];
        let expected_transform = Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0))
            * Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(second_node.transform, expected_transform);
        assert!(second_node.nodes.is_empty());
    }
}

#[derive(Clone, Debug)]
pub enum Asset {
    Model(Model),
    Mesh(Mesh),
    Texture(Texture),
}

pub struct AssetManager {
    asset_cache: HashMap<PathBuf, Arc<Asset>>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            asset_cache: HashMap::new(),
        }
    }

    pub fn get_asset_by_path(&mut self, path: &Path) -> Option<(Uuid, Arc<Asset>)> {
        if let Some(asset) = self.asset_cache.get(path) {
            Some((Uuid::nil(), Arc::clone(asset)))
        } else {
            if let Some(file) = ASSET_DIR.get_file(path) {
                let (gltf, buffers, images) = gltf::import_slice(file.contents()).ok()?;
                let model = AssetManager::gltf_to_model(gltf, buffers, images);

                let model_arc = Arc::new(Asset::Model(model));
                self.asset_cache
                    .insert(path.to_path_buf(), model_arc.clone());
                Some((Uuid::nil(), model_arc))
            } else {
                log::info!("file not found");

                None
            }
        }
    }

    pub fn gltf_to_model(
        gltf: Document,
        buffers: Vec<gltf::buffer::Data>,
        images: Vec<gltf::image::Data>,
    ) -> Model {
        let nodes = gltf
            .nodes()
            .map(|node| AssetManager::gltf_node_to_model_node(&node, &gltf, &buffers, &images))
            .collect();

        let materials = gltf
            .materials()
            .map(|mat| {
                let albedo_texture_index = mat
                    .pbr_metallic_roughness()
                    .base_color_texture()
                    .unwrap()
                    .texture()
                    .source()
                    .index();
                let albedo_image = images.get(albedo_texture_index).unwrap();
                let albedo_format = match albedo_image.format {
                    gltf::image::Format::R8G8B8 => ImageFormat::R8G8B8,
                    gltf::image::Format::R8G8B8A8 => ImageFormat::R8G8B8A8,
                    _ => panic!("unsupported image format"),
                };
                let albedo = Texture {
                    texture_type: TextureType::Albedo,
                    image_format: albedo_format,
                    width: albedo_image.width,
                    height: albedo_image.height,
                    data: albedo_image.pixels.clone(),
                };

                let normals = {
                    if let Some(normal_texture) = mat.normal_texture() {
                        let index = normal_texture.texture().source().index();
                        let image: &gltf::image::Data = images.get(index).unwrap();
                        let normals = Texture {
                            texture_type: TextureType::Normal,
                            image_format: ImageFormat::R8G8B8,
                            width: image.width,
                            height: image.height,
                            data: image.pixels.clone(),
                        };
                        Some(normals)
                    } else {
                        None
                    }
                };

                Material { albedo, normals }
            })
            .collect();

        Model { nodes, materials }
    }

    /// a recursive function that turns every gltf node into a ```ModelNode```
    fn gltf_node_to_model_node(
        node: &gltf::Node,
        gltf: &Document,
        buffers: &Vec<gltf::buffer::Data>,
        images: &Vec<gltf::image::Data>,
    ) -> ModelNode {
        let transform = Mat4::from_cols_array_2d(&node.transform().matrix());

        let meshes = gltf
            .meshes()
            .map(|mesh_data| {
                let primitives = mesh_data
                    .primitives()
                    .map(|prim| {
                        let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));
                        let tex_coords = match reader.read_tex_coords(0) {
                            Some(tcs) => tcs.into_f32().map(|tc| Vec2::from_array(tc)).collect(),
                            None => Vec::new(),
                        };
                        let mesh_primitive = MeshPrimitive {
                            positions: reader
                                .read_positions()
                                .unwrap()
                                .map(|p| Vec3::from_array(p))
                                .collect(),
                            normals: reader
                                .read_normals()
                                .unwrap()
                                .map(|n| Vec3::from_array(n))
                                .collect(),
                            tex_coords,
                            indices: reader.read_indices().unwrap().into_u32().collect(),
                            material_index: prim.material().index(),
                        };

                        mesh_primitive
                    })
                    .collect();

                let mesh = Mesh { primitives };

                mesh
            })
            .collect();

        let nodes = node
            .children()
            .map(|n| AssetManager::gltf_node_to_model_node(&n, gltf, buffers, images))
            .collect();

        ModelNode {
            transform,
            meshes,
            nodes,
        }
    }

    pub fn get_asset_by_id(&mut self, id: Uuid) -> Asset {
        todo!()
    }
}
