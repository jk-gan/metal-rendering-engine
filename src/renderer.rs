use crate::camera::{ArcballCamera, CameraFunction};
use crate::shader_bindings::Uniforms;
use glam::{Mat4, Vec3};
use metal::*;
use std::mem;
use tobj;

pub struct Renderer {
    pub device: Device,
    command_queue: CommandQueue,
    library: Library,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    pipeline_state: RenderPipelineState,
    uniforms: [Uniforms; 1],
    camera: ArcballCamera,
}

impl Renderer {
    pub fn new() -> Self {
        let device = Device::system_default().expect("GPU not available!");
        let command_queue = device.new_command_queue();

        let (mut models, _materials) = tobj::load_obj(
            "resources/teapot.obj",
            // &tobj::LoadOptions {
            //     triangulate: true,
            //     ..Default::default()
            // },
            &tobj::LoadOptions::default(),
        )
        .expect("Failed to load .obj file");

        let mut camera = ArcballCamera::new(0.5, 10.0, Vec3::new(0.0, 0.5, 0.0), 2.0);
        camera.set_rotation(Vec3::new(-10.0_f32.to_radians(), 0.0, 0.0));

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

        let library_path =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders/shaders.metallib");

        let library = device.new_library_with_file(library_path).unwrap();
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

        let pipeline_state = device
            .new_render_pipeline_state(&pipeline_state_descriptor)
            .unwrap();

        let translation = Mat4::from_translation(Vec3::new(0.0, 0.3, 0.0));
        let rotation = Mat4::from_rotation_y(10.0_f32.to_radians());
        let aspect_ratio = 800.0 / 800.0;
        let projection_matrix =
            Mat4::perspective_lh(45.0_f32.to_radians(), aspect_ratio, 0.1, 100.0);
        let uniforms = Uniforms {
            modelMatrix: unsafe { std::mem::transmute(translation * rotation) },
            viewMatrix: unsafe {
                std::mem::transmute(Mat4::from_translation(Vec3::new(0.0, 0.0, -3.0)).inverse())
            },
            projectionMatrix: unsafe { std::mem::transmute(projection_matrix) },
        };

        Self {
            device,
            command_queue,
            library,
            vertex_buffer,
            index_buffer,
            pipeline_state,
            uniforms: [uniforms],
            camera,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let aspect_ratio = width as f32 / height as f32;
        let projection_matrix =
            Mat4::perspective_lh(70.0_f32.to_radians(), aspect_ratio, 0.001, 100.0);
        self.uniforms[0].projectionMatrix = unsafe { std::mem::transmute(projection_matrix) };
    }

    pub fn zoom(&mut self, delta: f32) {
        self.camera.zoom(delta);
    }

    pub fn rotate(&mut self, delta: (f64, f64)) {
        self.camera.rotate((delta.0 as f32, delta.1 as f32));
    }

    pub fn draw(&mut self, drawable: &MetalDrawableRef) {
        let render_pass_descriptor = RenderPassDescriptor::new();

        let color_attachment = render_pass_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();

        color_attachment.set_texture(Some(&drawable.texture()));
        color_attachment.set_load_action(MTLLoadAction::Clear);
        // color_attachment.set_clear_color(MTLClearColor::new(0.2, 0.2, 0.25, 1.0));
        color_attachment.set_clear_color(MTLClearColor::new(1.0, 1.0, 1.0, 1.0));
        color_attachment.set_store_action(MTLStoreAction::Store);

        self.uniforms[0].projectionMatrix =
            unsafe { std::mem::transmute(*self.camera.projection_matrix()) };
        self.uniforms[0].viewMatrix = unsafe { std::mem::transmute(*self.camera.view_matrix()) };

        let command_buffer = self.command_queue.new_command_buffer();
        let render_encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);
        render_encoder.set_vertex_buffer(0, Some(&self.vertex_buffer), 0);
        render_encoder.set_vertex_bytes(
            1,
            std::mem::size_of::<Uniforms>() as u64,
            self.uniforms.as_ptr() as *const _,
        );

        render_encoder.set_render_pipeline_state(&self.pipeline_state);

        // render_encoder.set_triangle_fill_mode(MTLTriangleFillMode::Lines);
        render_encoder.draw_indexed_primitives(
            MTLPrimitiveType::Triangle,
            self.index_buffer.length(),
            MTLIndexType::UInt32,
            &self.index_buffer,
            0,
        );

        // render_encoder.draw_primitives_instanced(MTLPrimitiveType::Triangle, 0, 3, 1);
        render_encoder.end_encoding();

        command_buffer.present_drawable(&drawable);
        command_buffer.commit();
    }
}
