use crate::{node::Node, texturable::Texturable};
use glam::{Mat4, Vec3};
use metal::*;
use std::mem;
use tobj;

pub struct Submesh {
    // mesh: tobj::Mesh,
    vertices: Vec<f32>,
    pub(crate) indices: Vec<u32>,
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
    pub(crate) normal_buffer: Buffer,
}

pub struct Model {
    node: Node,
    pub(crate) submeshes: Vec<Submesh>,
    pub(crate) pipeline_state: RenderPipelineState,
}

impl Model {
    pub fn new(
        node: Node,
        submeshes: Vec<Submesh>,
        pipeline_state: RenderPipelineState,
    ) -> Model {
        Model {
            node,
            submeshes,
            pipeline_state,
        }
    }

    pub fn from_obj_filename(name: &str, device: &Device, library: &Library) -> Model {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("resources/{}", name));
        let (mut models, materials) = tobj::load_obj(
            path.as_path(),
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ignore_points: true,
                ignore_lines: true,
                ..Default::default()
            },
            // &tobj::LoadOptions::default(),
        )
        .expect(format!("Failed to load {} file", name).as_str());

        // let materials = materials.expect("Failed to load MTL file");
        let mut submeshes: Vec<Submesh> = vec![];

        for model in models {
            let mesh = model.mesh;
            let vertices = mesh.positions;
            let indices = mesh.indices;
            let normals = mesh.normals;

            let vertex_buffer = device.new_buffer_with_data(
                vertices.as_ptr() as *const _,
                mem::size_of::<f32>() as u64 * vertices.len() as u64,
                MTLResourceOptions::CPUCacheModeDefaultCache,
            );
            let index_buffer = device.new_buffer_with_data(
                indices.as_ptr() as *const _,
                mem::size_of::<u32>() as u64 * indices.len() as u64,
                MTLResourceOptions::CPUCacheModeDefaultCache,
            );
            let normal_buffer = device.new_buffer_with_data(
                normals.as_ptr() as *const _,
                mem::size_of::<f32>() as u64 * normals.len() as u64,
                MTLResourceOptions::CPUCacheModeDefaultCache,
            );

            let submesh = Submesh {
                vertices,
                indices,
                // mesh,
                vertex_buffer,
                index_buffer,
                normal_buffer,
            };
            submeshes.push(submesh);
        }

        let pipeline_state = Model::build_pipeline_state(library, device);

        let mut node = Node::default();
        node.name = name.to_string();

        Model::new(
            node,
            submeshes,
            pipeline_state,
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

        let color_attachment = pipeline_state_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        color_attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);

        device
            .new_render_pipeline_state(&pipeline_state_descriptor)
            .unwrap()
    }
}

fn default_vertex_descriptor() -> &'static VertexDescriptorRef {
    let vertex_descriptor = VertexDescriptor::new();

    let attribute_0 = vertex_descriptor.attributes().object_at(0).unwrap();
    attribute_0.set_format(MTLVertexFormat::Float3);
    attribute_0.set_offset(0);
    attribute_0.set_buffer_index(0);

    let attribute_1 = vertex_descriptor.attributes().object_at(1).unwrap();
    attribute_1.set_format(MTLVertexFormat::Float3);
    attribute_1.set_offset(0);
    attribute_1.set_buffer_index(1);
    // offset += mem::size_of::<f32>() as u64 * 3;

    let attribute_2 = vertex_descriptor.attributes().object_at(2).unwrap();
    attribute_2.set_format(MTLVertexFormat::Float2);
    attribute_2.set_offset(0);
    attribute_2.set_buffer_index(2);

    let layout_0 = vertex_descriptor.layouts().object_at(0).unwrap();
    layout_0.set_stride(mem::size_of::<f32>() as u64 * 3);
    let layout_1 = vertex_descriptor.layouts().object_at(1).unwrap();
    layout_1.set_stride(mem::size_of::<f32>() as u64 * 3);
    let layout_2 = vertex_descriptor.layouts().object_at(2).unwrap();
    layout_2.set_stride(mem::size_of::<f32>() as u64 * 2);

    vertex_descriptor
}
