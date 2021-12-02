use crate::camera::{ArcballCamera, CameraFunction};
use crate::model::Model;
use crate::shader_bindings::Uniforms;
use glam::{Mat4, Vec3};
use metal::*;

pub struct Renderer {
    pub device: Device,
    command_queue: CommandQueue,
    library: Library,
    uniforms: [Uniforms; 1],
    camera: ArcballCamera,
    models: Vec<Model>,
}

impl Renderer {
    pub fn new() -> Self {
        let device = Device::system_default().expect("GPU not available!");
        let command_queue = device.new_command_queue();

        let mut camera = ArcballCamera::new(0.5, 10.0, Vec3::new(0.0, 0.5, 0.0), 2.0);
        camera.set_rotation(Vec3::new(-10.0_f32.to_radians(), 0.0, 0.0));

        let library_path =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders/shaders.metallib");
        let library = device.new_library_with_file(library_path).unwrap();

        let model = Model::from_obj_filename("teapot.obj", &device, &library);
        let models = vec![model];

        let translation = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let rotation = Mat4::from_rotation_y(45.0_f32.to_radians());
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
            uniforms: [uniforms],
            camera,
            models,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let aspect_ratio = width as f32 / height as f32;
        self.camera.set_aspect_ratio(aspect_ratio);
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
        color_attachment.set_clear_color(MTLClearColor::new(0.2, 0.2, 0.25, 1.0));
        // color_attachment.set_clear_color(MTLClearColor::new(1.0, 1.0, 1.0, 1.0));
        color_attachment.set_store_action(MTLStoreAction::Store);

        let translation = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let rotation = Mat4::from_rotation_y(45.0_f32.to_radians());
        let scale = Mat4::from_scale(Vec3::new(1.0, 1.0, 1.0));
        self.uniforms[0].modelMatrix =
            unsafe { std::mem::transmute(translation * rotation * scale) };
        self.uniforms[0].projectionMatrix =
            unsafe { std::mem::transmute(*self.camera.projection_matrix()) };
        self.uniforms[0].viewMatrix = unsafe { std::mem::transmute(*self.camera.view_matrix()) };

        let command_buffer = self.command_queue.new_command_buffer();
        let render_encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);

        for model in self.models.iter() {
            render_encoder.set_vertex_buffer(0, Some(&model.vertex_buffer), 0);
            render_encoder.set_vertex_bytes(
                1,
                std::mem::size_of::<Uniforms>() as u64,
                self.uniforms.as_ptr() as *const _,
            );

            render_encoder.set_render_pipeline_state(&model.pipeline_state);

            render_encoder.set_triangle_fill_mode(MTLTriangleFillMode::Lines);
            render_encoder.draw_indexed_primitives(
                MTLPrimitiveType::Triangle,
                model.index_buffer.length(),
                MTLIndexType::UInt32,
                &model.index_buffer,
                0,
            );
        }

        // render_encoder.draw_primitives_instanced(MTLPrimitiveType::Triangle, 0, 3, 1);
        render_encoder.end_encoding();

        command_buffer.present_drawable(&drawable);
        command_buffer.commit();
    }
}
