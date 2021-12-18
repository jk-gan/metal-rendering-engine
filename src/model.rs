use crate::shader_bindings::{Attributes_Normal, Attributes_Position, Attributes_UV};
use crate::{node::Node, texturable::Texturable};
use glam::{Mat4, Vec3};
use metal::*;
use std::mem;
use tobj;

pub struct Submesh {
    // mesh: tobj::Mesh,
    pub(crate) material: Option<tobj::Material>,
    vertices: Vec<f32>,
    pub(crate) indices: Vec<u32>,
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
    pub(crate) normal_buffer: Buffer,
    pub(crate) text_coords_buffer: Buffer,
    pub(crate) textures: Option<Textures>,
}

impl Submesh {
    pub fn new(
        device: &Device,
        material: Option<tobj::Material>,
        vertices: Vec<f32>,
        indices: Vec<u32>,
        normals: Vec<f32>,
        text_coords: Vec<f32>,
        // vertex_buffer: Buffer,
        // index_buffer: Buffer,
        // normal_buffer: Buffer,
    ) -> Self {
        let textures = match material {
            Some(ref material) => {
                let texture_filename = &material.diffuse_texture;
                println!("texture_filename: {}", texture_filename);
                let texture =
                    Self::load_texture(texture_filename, &device).expect("Unable to load texture");
                let textures = Textures::new(material, texture);
                Some(textures)
            }
            None => None,
        };
        let vertex_buffer = device.new_buffer_with_data(
            vertices.as_ptr() as *const _,
            mem::size_of::<f32>() as u64 * vertices.len() as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
        );
        let index_buffer = device.new_buffer_with_data(
            indices.as_ptr() as *const _,
            mem::size_of::<u32>() as u64 * indices.len() as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
        );
        let normal_buffer = device.new_buffer_with_data(
            normals.as_ptr() as *const _,
            mem::size_of::<f32>() as u64 * normals.len() as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
        );
        let text_coords_buffer = device.new_buffer_with_data(
            text_coords.as_ptr() as *const _,
            mem::size_of::<f32>() as u64 * text_coords.len() as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
        );

        Self {
            material,
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            normal_buffer,
            text_coords_buffer,
            textures,
        }
    }
}

impl Texturable for Submesh {}

pub struct Model {
    node: Node,
    pub(crate) submeshes: Vec<Submesh>,
    pub(crate) pipeline_state: RenderPipelineState,
    pub(crate) tiling: u32,
    pub(crate) sampler_state: SamplerState,
}

impl Model {
    pub fn new(
        node: Node,
        submeshes: Vec<Submesh>,
        pipeline_state: RenderPipelineState,
        tiling: u32,
        sampler_state: SamplerState,
    ) -> Model {
        Model {
            node,
            submeshes,
            pipeline_state,
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
            let mesh = model.mesh;
            let vertices = mesh.positions;
            let indices = mesh.indices;
            let normals = mesh.normals;
            let text_coords = mesh.texcoords;
            let mut material = None;
            if let Some(id) = mesh.material_id {
                material = match materials {
                    Some(ref materials) => Some(materials[id].clone()),
                    None => None,
                };
            }

            let submesh = Submesh::new(&device, material, vertices, indices, normals, text_coords);
            submeshes.push(submesh);
        }

        let pipeline_state = Model::build_pipeline_state(library, device);
        let sampler_state = Model::build_sampler_state(device);

        let mut node = Node::default();
        node.name = name.to_string();

        Model::new(node, submeshes, pipeline_state, tiling, sampler_state)
    }

        )
    }

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

    let attribute_0 = vertex_descriptor.attributes().object_at(0).unwrap();
    attribute_0.set_format(MTLVertexFormat::Float3);
    attribute_0.set_offset(0);
    attribute_0.set_buffer_index(Attributes_Position as u64);

    let attribute_1 = vertex_descriptor.attributes().object_at(1).unwrap();
    attribute_1.set_format(MTLVertexFormat::Float3);
    attribute_1.set_offset(0);
    attribute_1.set_buffer_index(Attributes_Normal as u64);
    // offset += mem::size_of::<f32>() as u64 * 3;

    let attribute_2 = vertex_descriptor.attributes().object_at(2).unwrap();
    attribute_2.set_format(MTLVertexFormat::Float2);
    attribute_2.set_offset(0);
    attribute_2.set_buffer_index(Attributes_UV as u64);

    let layout_0 = vertex_descriptor.layouts().object_at(0).unwrap();
    layout_0.set_stride(mem::size_of::<f32>() as u64 * 3);
    let layout_1 = vertex_descriptor.layouts().object_at(1).unwrap();
    layout_1.set_stride(mem::size_of::<f32>() as u64 * 3);
    let layout_2 = vertex_descriptor.layouts().object_at(2).unwrap();
    layout_2.set_stride(mem::size_of::<f32>() as u64 * 2);

    vertex_descriptor
}

pub struct Textures {
    // filename: String,
    pub(crate) diffuse_texture: Texture,
}

impl Textures {
    fn new(material: &tobj::Material, texture: Texture) -> Textures {
        // let filename = material.diffuse_texture.clone();

        Textures {
            // filename,
            diffuse_texture: texture,
        }
    }
}
