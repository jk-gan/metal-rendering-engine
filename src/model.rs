use crate::node::Node;
use metal::*;
use std::mem;
use tobj;

pub struct Model {
    node: Node,
    vertices: Vec<f32>,
    indices: Vec<u32>,
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
    pub(crate) pipeline_state: RenderPipelineState,
}

impl Model {
    pub fn new(
        node: Node,
        vertices: Vec<f32>,
        indices: Vec<u32>,
        vertex_buffer: Buffer,
        index_buffer: Buffer,
        pipeline_state: RenderPipelineState,
    ) -> Model {
        Model {
            node,
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            pipeline_state,
        }
    }

    pub fn from_obj_filename(name: &str, device: &Device, library: &Library) -> Model {
        let (mut models, _materials) = tobj::load_obj(
            format!("resources/{}", name),
            // "resources/teapot.obj",
            // &tobj::LoadOptions {
            //     triangulate: true,
            //     ..Default::default()
            // },
            &tobj::LoadOptions::default(),
        )
        .expect(format!("Failed to load {} file", name).as_str());

        // let materials = materials.expect("Failed to load MTL file");
        let first_model = models.pop().unwrap();
        let mesh = first_model.mesh;
        let vertices = mesh.positions;
        let indices = mesh.indices;

        let vertex_buffer = device.new_buffer_with_data(
            vertices.as_ptr() as *const _,
            mem::size_of::<f32>() as u64 * 3 * vertices.len() as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let index_buffer = device.new_buffer_with_data(
            indices.as_ptr() as *const _,
            mem::size_of::<u32>() as u64 * indices.len() as u64,
            MTLResourceOptions::StorageModeShared,
        );

        let pipeline_state = Model::build_pipeline_state(library, device);

        let mut node = Node::default();
        node.name = name.to_string();

        Model::new(
            node,
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            pipeline_state,
        )
    }

    fn build_pipeline_state(library: &Library, device: &Device) -> RenderPipelineState {
        let vertex_function = library.get_function("vertex_main", None).unwrap();
        let fragment_function = library.get_function("fragment_main", None).unwrap();

        let vertex_descriptor = VertexDescriptor::new();
        let attribute = vertex_descriptor.attributes().object_at(0).unwrap();
        attribute.set_format(MTLVertexFormat::Float3);
        attribute.set_offset(0);
        attribute.set_buffer_index(0);
        let layout = vertex_descriptor.layouts().object_at(0).unwrap();
        layout.set_stride(mem::size_of::<f32>() as u64 * 3);

        let pipeline_state_descriptor = RenderPipelineDescriptor::new();
        pipeline_state_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_state_descriptor.set_fragment_function(Some(&fragment_function));
        pipeline_state_descriptor.set_vertex_descriptor(Some(&vertex_descriptor));

        let color_attachment = pipeline_state_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        color_attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        // pipeline_state_descriptor.set_depth_attachment_pixel_format(MTLPixelFormat::Invalid);

        device
            .new_render_pipeline_state(&pipeline_state_descriptor)
            .unwrap()
    }
}
