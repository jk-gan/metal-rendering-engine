use crate::shader_bindings::{
    Attributes_Normal, Attributes_Position, Attributes_UV,
    BufferIndices_BufferIndexVertices as BufferIndexVertices,
};
use crate::{node::Node, texturable::Texturable};
use glam::{Mat4, Vec3};
use gltf::Gltf;
use metal::*;
use std::mem;
use tobj;

pub struct ModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub text_coords: [f32; 2],
}

pub struct Submesh {
    // mesh: tobj::Mesh,
    // pub(crate) material: Option<tobj::Material>,
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
    pub(crate) num_elements: u64,
    pub(crate) textures: Option<Textures>,
    pub(crate) pipeline_state: RenderPipelineState,
}

impl Submesh {
    pub fn new(
        device: &Device,
        library: &Library,
        material: Option<tobj::Material>,
        vertex_buffer: Buffer,
        index_buffer: Buffer,
        num_elements: u64,
    ) -> Self {
        let textures = match material {
            Some(ref material) => {
                let diffuse_texture = match &material.diffuse_texture {
                    x if x.is_empty() => None,
                    filename => {
                        println!("diffuse_texture_filename: {}", filename);
                        let diffuse_texture =
                            Self::load_texture(filename, &device).expect("Unable to load texture");
                        Some(diffuse_texture)
                    }
                };

                let normal_texture = match &material.normal_texture {
                    x if x.is_empty() => None,
                    filename => {
                        println!("diffuse_texture_filename: {}", filename);
                        let normal_texture =
                            Self::load_texture(filename, &device).expect("Unable to load texture");
                        Some(normal_texture)
                    }
                };

                let textures = Textures::new(diffuse_texture, normal_texture);
                Some(textures)
            }
            None => None,
        };

        let pipeline_state = Submesh::build_pipeline_state(library, device);

        Self {
            vertex_buffer,
            index_buffer,
            num_elements,
            textures,
            pipeline_state,
        }
    }

    // pub fn from_gltf(
    //     device: &Device,
    //     library: &Library,
    //     texture_source: &str,
    //     vertices: Vec<f32>,
    //     indices: Vec<u32>,
    //     normals: Vec<f32>,
    //     text_coords: Vec<f32>,
    // ) -> Self {
    //     // let textures = match material {
    //     //     Some(ref material) => {
    //     //         let texture_filename = &material.diffuse_texture;
    //     //         println!("texture_filename: {}", texture_filename);
    //     //         let texture =
    //     //             Self::load_texture(texture_filename, &device).expect("Unable to load texture");
    //     //         let textures = Textures::new(material, texture);
    //     //         Some(textures)
    //     //     }
    //     //     None => None,
    //     // };

    //     println!("texture_source: {}", texture_source);
    //     let texture = Self::load_texture(format!("adamHead/{}", texture_source).as_ref(), &device)
    //         .expect("Unable to load texture");
    //     let normal_texture =
    //         Self::load_texture(format!("adamHead/{}", texture_source).as_ref(), &device)
    //             .expect("Unable to load texture");
    //     let textures = Some(Textures::new(Some(texture), Some(normal_texture)));

    //     let vertex_buffer = device.new_buffer_with_data(
    //         vertices.as_ptr() as *const _,
    //         mem::size_of::<f32>() as u64 * vertices.len() as u64,
    //         MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
    //     );
    //     let index_buffer = device.new_buffer_with_data(
    //         indices.as_ptr() as *const _,
    //         mem::size_of::<u32>() as u64 * indices.len() as u64,
    //         MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
    //     );
    //     let normal_buffer = device.new_buffer_with_data(
    //         normals.as_ptr() as *const _,
    //         mem::size_of::<f32>() as u64 * normals.len() as u64,
    //         MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
    //     );
    //     let text_coords_buffer = device.new_buffer_with_data(
    //         text_coords.as_ptr() as *const _,
    //         mem::size_of::<f32>() as u64 * text_coords.len() as u64,
    //         MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
    //     );

    //     let pipeline_state = Submesh::build_pipeline_state(library, device);

    //     Self {
    //         vertices,
    //         indices,
    //         vertex_buffer,
    //         index_buffer,
    //         normal_buffer,
    //         text_coords_buffer,
    //         textures,
    //         pipeline_state,
    //     }
    // }

    fn build_pipeline_state(library: &Library, device: &Device) -> RenderPipelineState {
        let vertex_function = library.get_function("vertex_main", None).unwrap();
        let fragment_function = library.get_function("fragment_main", None).unwrap();
        let vertex_descriptor = default_vertex_descriptor();

        let pipeline_state_descriptor = RenderPipelineDescriptor::new();
        pipeline_state_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_state_descriptor.set_fragment_function(Some(&fragment_function));
        pipeline_state_descriptor.set_vertex_descriptor(Some(&vertex_descriptor));
        pipeline_state_descriptor.set_depth_attachment_pixel_format(MTLPixelFormat::Depth32Float);
        pipeline_state_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap()
            .set_pixel_format(MTLPixelFormat::BGRA8Unorm);

        device
            .new_render_pipeline_state(&pipeline_state_descriptor)
            .unwrap()
    }
}

impl Texturable for Submesh {}

pub struct Model {
    node: Node,
    pub(crate) submeshes: Vec<Submesh>,
    pub(crate) tiling: u32,
    pub(crate) sampler_state: SamplerState,
}

impl Model {
    pub fn new(
        node: Node,
        submeshes: Vec<Submesh>,
        tiling: u32,
        sampler_state: SamplerState,
    ) -> Model {
        Model {
            node,
            submeshes,
            tiling,
            sampler_state,
        }
    }

    pub fn from_obj_filename(name: &str, tiling: u32, device: &Device, library: &Library) -> Model {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("resources/{}", name));
        let (models, materials) = tobj::load_obj(
            path.as_path(),
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                // ignore_points: true,
                // ignore_lines: true,
                ..Default::default()
            },
            // &tobj::LoadOptions::default(),
        )
        .expect(format!("Failed to load {} file", name).as_str());

        let materials = match materials {
            Ok(materials) => Some(materials),
            Err(_e) => {
                println!("Failed to load {} file", name);
                None
            }
        };

        let mut submeshes: Vec<Submesh> = vec![];

        for model in models {
            let indices = model.mesh.indices;
            let mut material = None;
            if let Some(id) = model.mesh.material_id {
                material = match materials {
                    Some(ref materials) => Some(materials[id].clone()),
                    None => None,
                };
            }

            let mut vertices = Vec::new();
            for i in 0..model.mesh.positions.len() / 3 {
                vertices.push(ModelVertex {
                    position: [
                        model.mesh.positions[i * 3],
                        model.mesh.positions[i * 3 + 1],
                        model.mesh.positions[i * 3 + 2],
                    ],
                    normal: [
                        model.mesh.normals[i * 3],
                        model.mesh.normals[i * 3 + 1],
                        model.mesh.normals[i * 3 + 2],
                    ],
                    text_coords: [model.mesh.texcoords[i * 2], model.mesh.texcoords[i * 2 + 1]],
                });
            }

            let vertex_buffer = device.new_buffer_with_data(
                vertices.as_ptr() as *const _,
                mem::size_of::<ModelVertex>() as u64 * vertices.len() as u64,
                MTLResourceOptions::CPUCacheModeDefaultCache
                    | MTLResourceOptions::StorageModeManaged,
            );
            let index_buffer = device.new_buffer_with_data(
                indices.as_ptr() as *const _,
                mem::size_of::<u32>() as u64 * indices.len() as u64,
                MTLResourceOptions::CPUCacheModeDefaultCache
                    | MTLResourceOptions::StorageModeManaged,
            );
            let num_elements = indices.len() as u64;

            let submesh = Submesh::new(
                &device,
                &library,
                material,
                vertex_buffer,
                index_buffer,
                num_elements,
            );
            submeshes.push(submesh);
        }

        // let pipeline_state = Model::build_pipeline_state(library, device);
        let sampler_state = Model::build_sampler_state(device);

        let mut node = Node::default();
        node.name = name.to_string();

        Model::new(node, submeshes, tiling, sampler_state)
    }

    // pub fn from_gltf_filename(
    //     name: &str,
    //     tiling: u32,
    //     device: &Device,
    //     library: &Library,
    // ) -> Model {
    //     let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //         .join(format!("resources/{}", name));
    //     let (gltf, buffers, _) = gltf::import(path.as_path()).expect("Failed to load gltf file");

    //     let mut submeshes: Vec<Submesh> = vec![];

    //     // for model in models {
    //     //     let mesh = model.mesh;
    //     //     let vertices = mesh.positions;
    //     //     let indices = mesh.indices;
    //     //     let normals = mesh.normals;
    //     //     let text_coords = mesh.texcoords;
    //     //     let mut material = None;
    //     //     if let Some(id) = mesh.material_id {
    //     //         material = match materials {
    //     //             Some(ref materials) => Some(materials[id].clone()),
    //     //             None => None,
    //     //         };
    //     //     }

    //     //     let submesh = Submesh::new(&device, material, vertices, indices, normals, text_coords);
    //     //     submeshes.push(submesh);
    //     // }

    //     for mesh in gltf.meshes() {
    //         println!("Mesh #{}", mesh.index());
    //         for primitive in mesh.primitives() {
    //             println!("- Primitive #{}", primitive.index());
    //             let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    //             let mut vertices = vec![];
    //             let mut indices = vec![];
    //             let mut normals = vec![];
    //             let mut text_coords = vec![];

    //             if let Some(iter) = reader.read_positions() {
    //                 for vertex_position in iter {
    //                     vertices.push(vertex_position[0]);
    //                     vertices.push(vertex_position[1]);
    //                     vertices.push(vertex_position[2]);
    //                 }
    //             }

    //             if let Some(iter) = reader.read_indices() {
    //                 match iter {
    //                     gltf::mesh::util::ReadIndices::U8(iter) => {
    //                         for index in iter {
    //                             indices.push(index as u32);
    //                         }
    //                     }
    //                     gltf::mesh::util::ReadIndices::U16(iter) => {
    //                         for index in iter {
    //                             indices.push(index as u32);
    //                         }
    //                     }
    //                     gltf::mesh::util::ReadIndices::U32(iter) => {
    //                         for index in iter {
    //                             indices.push(index);
    //                         }
    //                     }
    //                 };
    //             }

    //             if let Some(iter) = reader.read_normals() {
    //                 for vertex_normal in iter {
    //                     normals.push(vertex_normal[0]);
    //                     normals.push(vertex_normal[1]);
    //                     normals.push(vertex_normal[2]);
    //                 }
    //             }

    //             if let Some(iter) = reader.read_tex_coords(1) {
    //                 match iter {
    //                     gltf::mesh::util::ReadTexCoords::U8(iter) => {
    //                         for text_coord in iter {
    //                             text_coords.push(text_coord[0] as f32);
    //                             text_coords.push(text_coord[1] as f32);
    //                         }
    //                     }
    //                     gltf::mesh::util::ReadTexCoords::U16(iter) => {
    //                         for text_coord in iter {
    //                             text_coords.push(text_coord[0] as f32);
    //                             text_coords.push(text_coord[1] as f32);
    //                         }
    //                     }
    //                     gltf::mesh::util::ReadTexCoords::F32(iter) => {
    //                         for text_coord in iter {
    //                             text_coords.push(text_coord[0]);
    //                             text_coords.push(text_coord[1]);
    //                         }
    //                     }
    //                 }
    //             }

    //             let texture_source =
    //                 match gltf.textures().nth(mesh.index()).unwrap().source().source() {
    //                     gltf::image::Source::Uri { uri, .. } => {
    //                         uri
    //                         // let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //                         //     .join(format!("resources/{}", uri));
    //                         // let image = image::open(path).expect("Failed to load image");
    //                     }
    //                     _ => todo!(),
    //                 };

    //             let submesh = Submesh::from_gltf(
    //                 &device,
    //                 &library,
    //                 texture_source,
    //                 vertices,
    //                 indices,
    //                 normals,
    //                 text_coords,
    //             );
    //         }
    //     }

    //     let (models, materials) = tobj::load_obj(
    //         path.as_path(),
    //         &tobj::LoadOptions {
    //             triangulate: true,
    //             single_index: true,
    //             // ignore_points: true,
    //             // ignore_lines: true,
    //             ..Default::default()
    //         },
    //         // &tobj::LoadOptions::default(),
    //     )
    //     .expect(format!("Failed to load {} file", name).as_str());

    //     let materials = match materials {
    //         Ok(materials) => Some(materials),
    //         Err(_e) => {
    //             println!("Failed to load {} file", name);
    //             None
    //         }
    //     };

    //     let mut submeshes: Vec<Submesh> = vec![];

    //     for model in models {
    //         let mesh = model.mesh;
    //         let vertices = mesh.positions;
    //         let indices = mesh.indices;
    //         let normals = mesh.normals;
    //         let text_coords = mesh.texcoords;
    //         let mut material = None;
    //         if let Some(id) = mesh.material_id {
    //             material = match materials {
    //                 Some(ref materials) => Some(materials[id].clone()),
    //                 None => None,
    //             };
    //         }

    //         let submesh = Submesh::new(
    //             &device,
    //             &library,
    //             material,
    //             vertices,
    //             indices,
    //             normals,
    //             text_coords,
    //         );
    //         submeshes.push(submesh);
    //     }

    //     // let pipeline_state = Model::build_pipeline_state(library, device);
    //     let sampler_state = Model::build_sampler_state(device);

    //     let mut node = Node::default();
    //     node.name = name.to_string();

    //     Model::new(node, submeshes, tiling, sampler_state)
    // }

    pub fn set_position(&mut self, position: Vec3) {
        self.node.position = position;
    }

    pub fn set_rotation(&mut self, rotation: Vec3) {
        self.node.rotation = rotation;
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.node.scale = scale;
    }

    pub fn model_matrix(&self) -> Mat4 {
        self.node.model_matrix()
    }

    fn build_sampler_state(device: &Device) -> SamplerState {
        let descriptor = SamplerDescriptor::new();
        descriptor.set_address_mode_s(MTLSamplerAddressMode::Repeat);
        descriptor.set_address_mode_t(MTLSamplerAddressMode::Repeat);
        descriptor.set_mip_filter(MTLSamplerMipFilter::Linear);
        descriptor.set_max_anisotropy(8);
        device.new_sampler(&descriptor)
    }
}

fn default_vertex_descriptor() -> &'static VertexDescriptorRef {
    let vertex_descriptor = VertexDescriptor::new();
    let mut offset = 0;

    let attribute_0 = vertex_descriptor
        .attributes()
        .object_at(Attributes_Position as u64)
        .unwrap();
    attribute_0.set_format(MTLVertexFormat::Float3);
    attribute_0.set_offset(offset);
    attribute_0.set_buffer_index(BufferIndexVertices as u64);

    offset += mem::size_of::<f32>() as u64 * 3;

    let attribute_1 = vertex_descriptor
        .attributes()
        .object_at(Attributes_Normal as u64)
        .unwrap();
    attribute_1.set_format(MTLVertexFormat::Float3);
    attribute_1.set_offset(offset);
    attribute_1.set_buffer_index(BufferIndexVertices as u64);

    offset += mem::size_of::<f32>() as u64 * 3;

    let attribute_2 = vertex_descriptor
        .attributes()
        .object_at(Attributes_UV as u64)
        .unwrap();
    attribute_2.set_format(MTLVertexFormat::Float2);
    attribute_2.set_offset(offset);
    attribute_2.set_buffer_index(BufferIndexVertices as u64);

    offset += mem::size_of::<f32>() as u64 * 2;

    let layout_0 = vertex_descriptor.layouts().object_at(0).unwrap();
    layout_0.set_stride(offset);

    vertex_descriptor
}

pub struct Textures {
    // filename: String,
    pub(crate) diffuse_texture: Option<Texture>,
    pub(crate) normal_texture: Option<Texture>,
}

impl Textures {
    fn new(diffuse_texture: Option<Texture>, normal_texture: Option<Texture>) -> Textures {
        Textures {
            diffuse_texture,
            normal_texture,
        }
    }
}
