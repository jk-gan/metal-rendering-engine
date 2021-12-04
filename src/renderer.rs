use crate::camera::{ArcballCamera, CameraFunction};
use crate::model::Model;
use crate::shader_bindings::{
    Attributes_Normal, Attributes_Position,
    BufferIndices_BufferIndexFragmentUniforms as BufferIndexFragmentUniforms,
    BufferIndices_BufferIndexLights as BufferIndexLights,
    BufferIndices_BufferIndexUniforms as BufferIndexUniforms, FragementUniforms, Light,
    LightType_Ambientlight, LightType_Pointlight, LightType_Spotlight, LightType_Sunlight,
    Uniforms,
};
use glam::{Mat3A, Mat4, Vec3, Vec3A};
use metal::*;

pub struct Renderer {
    pub device: Device,
    command_queue: CommandQueue,
    library: Library,
    uniforms: [Uniforms; 1],
    fragment_uniforms: [FragementUniforms; 1],
    camera: ArcballCamera,
    models: Vec<Model>,
    depth_stencil_state: DepthStencilState,
    lights: Vec<Light>,
}

impl Renderer {
    pub fn new() -> Self {
        let device = Device::system_default().expect("GPU not available!");
        let command_queue = device.new_command_queue();

        let mut camera = ArcballCamera::new(0.5, 10.0, Vec3::new(0.0, 0.3, 0.0), 2.0);
        camera.set_rotation(Vec3::new(-10.0_f32.to_radians(), 0.0, 0.0));

        let library_path =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders/shaders.metallib");
        let library = device.new_library_with_file(library_path).unwrap();

        let mut model = Model::from_obj_filename("teapot.obj", &device, &library);
        model.set_position(Vec3::new(0.0, 0.0, 0.0));
        model.set_rotation(Vec3::new(0.0, 45.0_f32.to_radians(), 0.0));
        let models = vec![model];

        let uniforms = Uniforms {
            modelMatrix: unsafe { std::mem::transmute(Mat4::ZERO) },
            viewMatrix: unsafe { std::mem::transmute(Mat4::ZERO) },
            projectionMatrix: unsafe { std::mem::transmute(Mat4::ZERO) },
            normalMatrix: unsafe { std::mem::transmute(Mat3A::ZERO) },
        };

        let depth_stencil_state = Self::build_depth_stencil_state(&device);

        let sunlight = {
            let mut light = Self::build_default_light();
            light.position = unsafe { std::mem::transmute(Vec3A::new(1.0, 2.0, -2.0)) };
            light
        };

        let ambient_light = {
            let mut light = Self::build_default_light();
            light.color = unsafe { std::mem::transmute(Vec3A::new(0.5, 1.0, 0.0)) };
            light.intensity = 0.1;
            light.type_ = LightType_Ambientlight;
            light
        };

        let red_light = {
            let mut light = Self::build_default_light();
            light.position = unsafe { std::mem::transmute(Vec3A::new(-5.0, 1.5, -0.5)) };
            light.color = unsafe { std::mem::transmute(Vec3A::new(1.0, 0.0, 0.0)) };
            light.attenuation = unsafe { std::mem::transmute(Vec3A::new(1.0, 3.0, 4.0)) };
            light.type_ = LightType_Pointlight;
            light
        };

        let spotlight = {
            unsafe {
                let mut light = Self::build_default_light();
                light.position = std::mem::transmute(Vec3A::new(0.4, 0.8, 1.0));
                light.color = std::mem::transmute(Vec3A::new(1.0, 0.0, 1.0));
                light.attenuation = std::mem::transmute(Vec3A::new(1.0, 0.5, 0.0));
                light.type_ = LightType_Spotlight;
                light.coneAngle = 40.0_f32.to_radians();
                light.coneDirection = std::mem::transmute(Vec3A::new(-2.0, 0.0, -1.5));
                light.coneAttenuation = 12.0;
                light
            }
        };

        let mut lights: Vec<Light> = vec![];
        lights.push(sunlight);
        lights.push(ambient_light);
        // lights.push(red_light);
        lights.push(spotlight);

        let camera_position = camera.position();

        let fragment_uniforms = FragementUniforms {
            lightCount: lights.len() as u32,
            cameraPosition: unsafe {
                std::mem::transmute(Vec3A::new(
                    camera_position.x,
                    camera_position.y,
                    camera_position.z,
                ))
            },
            __bindgen_padding_0: unsafe { std::mem::zeroed() },
        };

        Self {
            device,
            command_queue,
            library,
            uniforms: [uniforms],
            fragment_uniforms: [fragment_uniforms],
            camera,
            models,
            depth_stencil_state,
            lights,
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

        let depth_buffer_descriptor = TextureDescriptor::new();
        depth_buffer_descriptor.set_width(3000);
        depth_buffer_descriptor.set_height(3000);
        depth_buffer_descriptor.set_pixel_format(MTLPixelFormat::Depth32Float);
        depth_buffer_descriptor.set_storage_mode(MTLStorageMode::Private);
        depth_buffer_descriptor
            .set_usage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
        let depth_attachment = render_pass_descriptor.depth_attachment().unwrap();
        depth_attachment.set_texture(Some(&self.device.new_texture(&depth_buffer_descriptor)));
        depth_attachment.set_load_action(MTLLoadAction::Clear);
        depth_attachment.set_store_action(MTLStoreAction::Store);
        depth_attachment.set_clear_depth(1.0);
        let stencil_attachment = render_pass_descriptor.stencil_attachment().unwrap();
        stencil_attachment.set_texture(depth_attachment.texture());

        self.uniforms[0].projectionMatrix =
            unsafe { std::mem::transmute(*self.camera.projection_matrix()) };
        self.uniforms[0].viewMatrix = unsafe { std::mem::transmute(*self.camera.view_matrix()) };

        let command_buffer = self.command_queue.new_command_buffer();
        let render_encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);
        render_encoder.set_depth_stencil_state(&self.depth_stencil_state);
        // render_encoder.set_front_facing_winding(MTLWinding::CounterClockwise);
        // render_encoder.set_cull_mode(MTLCullMode::Back);

        render_encoder.set_fragment_bytes(
            BufferIndexLights as u64,
            std::mem::size_of::<Light>() as u64 * self.lights.len() as u64,
            self.lights.as_ptr() as *const _,
        );
        render_encoder.set_fragment_bytes(
            BufferIndexFragmentUniforms as u64,
            std::mem::size_of::<FragementUniforms>() as u64,
            self.fragment_uniforms.as_ptr() as *const _,
        );

        for model in self.models.iter() {
            self.uniforms[0].modelMatrix = unsafe { std::mem::transmute(model.model_matrix()) };
            self.uniforms[0].normalMatrix =
                unsafe { std::mem::transmute(Mat3A::from_mat4(model.model_matrix())) };

            render_encoder.set_vertex_buffer(
                Attributes_Position as u64,
                Some(&model.vertex_buffer),
                0,
            );
            render_encoder.set_vertex_buffer(
                Attributes_Normal as u64,
                Some(&model.normal_buffer),
                0,
            );
            render_encoder.set_vertex_bytes(
                BufferIndexUniforms as u64,
                std::mem::size_of::<Uniforms>() as u64,
                self.uniforms.as_ptr() as *const _,
            );

            render_encoder.set_render_pipeline_state(&model.pipeline_state);

            // render_encoder.set_triangle_fill_mode(MTLTriangleFillMode::Lines);
            render_encoder.draw_indexed_primitives(
                MTLPrimitiveType::Triangle,
                model.indices.len() as u64,
                MTLIndexType::UInt32,
                &model.index_buffer,
                0,
            );
        }

        render_encoder.end_encoding();

        command_buffer.present_drawable(&drawable);
        command_buffer.commit();
    }

    fn build_default_light() -> Light {
        unsafe {
            Light {
                position: std::mem::transmute(Vec3A::new(0.0, 0.0, 0.0)),
                color: std::mem::transmute(Vec3A::new(1.0, 1.0, 1.0)),
                specularColor: std::mem::transmute(Vec3A::new(0.6, 0.6, 0.6)),
                intensity: 1.0,
                attenuation: std::mem::transmute(Vec3A::new(1.0, 0.0, 0.0)),
                type_: LightType_Sunlight,
                coneAngle: 0.0,
                coneDirection: std::mem::transmute(Vec3A::new(0.0, 0.0, 0.0)),
                coneAttenuation: 0.0,
                __bindgen_padding_0: std::mem::zeroed(),
                __bindgen_padding_1: std::mem::zeroed(),
            }
        }
    }

    fn build_depth_stencil_state(device: &Device) -> DepthStencilState {
        let descriptor = DepthStencilDescriptor::new();
        descriptor.set_depth_compare_function(MTLCompareFunction::Less);
        descriptor.set_depth_write_enabled(true);
        device.new_depth_stencil_state(&descriptor)
    }
}
