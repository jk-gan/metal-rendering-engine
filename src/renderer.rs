use crate::camera::{ArcballCamera, CameraFunction};
use crate::shader_bindings::{
    BufferIndices_BufferIndexFragmentUniforms as BufferIndexFragmentUniforms,
    BufferIndices_BufferIndexLights as BufferIndexLights,
    BufferIndices_BufferIndexUniforms as BufferIndexUniforms,
    BufferIndices_BufferIndexVertices as BufferIndexVertices, FragementUniforms, Light,
    LightType_Ambientlight, LightType_Pointlight, LightType_Spotlight, LightType_Sunlight,
    Textures_BaseColorTexture, Textures_NormalTexture, Uniforms,
};
use crate::{lighting::Lighting, model::Model};
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
    lighting: Lighting,
}

impl Renderer {
    pub fn new() -> Self {
        let device = Device::system_default().expect("GPU not available!");
        let command_queue = device.new_command_queue();

        let mut camera = ArcballCamera::new(0.5, 10.0, Vec3::new(0.0, 2.2, 0.0), 6.0);
        camera.set_rotation(Vec3::new(-10.0_f32.to_radians(), 0.0, 0.0));

        let library_path =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders/shaders.metallib");
        let library = device.new_library_with_file(library_path).unwrap();

        // Sofa
        // let mut model = Model::from_obj_filename("HepBurn_Sofa.obj", &device, &library);
        // model.set_position(Vec3::new(0.0, 0.0, 0.0));
        // model.set_rotation(Vec3::new(0.0, 180.0_f32.to_radians(), 0.0));
        // model.set_scale(Vec3::new(0.001, 0.001, 0.001));

        // let mut model = Model::from_obj_filename("lowpoly-house.obj", 1, &device, &library);
        // model.set_position(Vec3::new(0.0, 0.0, 0.0));
        // model.set_rotation(Vec3::new(0.0, 45.0_f32.to_radians(), 0.0));

        // let mut model = Model::from_obj_filename("train.obj", &device, &library);
        // let mut model = Model::from_obj_filename("viking_room.obj", &device, &library);
        // model.set_scale(Vec3::new(0.001, 0.001, 0.001));

        // let mut ground = Model::from_obj_filename("plane.obj", 16, &device, &library);
        // ground.set_scale(Vec3::new(40.0, 40.0, 40.0));

        let mut house = Model::from_obj_filename("cottage1.obj", 1, &device, &library);
        house.set_position(Vec3::new(0.0, 0.0, 0.0));
        house.set_rotation(Vec3::new(0.0, 50.0_f32.to_radians(), 0.0));

        // let adam_head = Model::from_gltf_filename("adamHead/adamHead.gltf", 1, &device, &library);

        let models = vec![house];

        // generate mipmaps
        for model in models.iter() {
            for submesh in model.submeshes.iter() {
                if let Some(textures) = &submesh.textures {
                    if let Some(texture) = &textures.diffuse_texture {
                        let command_buffer = command_queue.new_command_buffer();
                        let blit_command_encoder = command_buffer.new_blit_command_encoder();
                        blit_command_encoder.generate_mipmaps(&texture);
                        blit_command_encoder.end_encoding();
                        command_buffer.commit();
                    }
                }
            }
        }

        let uniforms = Uniforms {
            modelMatrix: unsafe { std::mem::transmute(Mat4::ZERO) },
            viewMatrix: unsafe { std::mem::transmute(Mat4::ZERO) },
            projectionMatrix: unsafe { std::mem::transmute(Mat4::ZERO) },
            normalMatrix: unsafe { std::mem::transmute(Mat3A::ZERO) },
        };

        let depth_stencil_state = Self::build_depth_stencil_state(&device);

        let lighting = Lighting::new();

        let camera_position = camera.position();

        let fragment_uniforms = FragementUniforms {
            lightCount: lighting.count,
            cameraPosition: unsafe {
                std::mem::transmute(Vec3A::new(
                    camera_position.x,
                    camera_position.y,
                    camera_position.z,
                ))
            },
            tiling: 1,
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
            lighting,
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
        // color_attachment.set_clear_color(MTLClearColor::new(0.2, 0.2, 0.25, 1.0));
        color_attachment.set_clear_color(MTLClearColor::new(0.93, 0.97, 1.0, 1.0));
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

        self.uniforms[0].projectionMatrix =
            unsafe { std::mem::transmute(*self.camera.projection_matrix()) };
        self.uniforms[0].viewMatrix = unsafe { std::mem::transmute(*self.camera.view_matrix()) };
        self.fragment_uniforms[0].cameraPosition = unsafe {
            std::mem::transmute(Vec3A::new(
                self.camera.position().x,
                self.camera.position().y,
                self.camera.position().z,
            ))
        };

        let command_buffer = self.command_queue.new_command_buffer();
        let render_encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);
        render_encoder.set_depth_stencil_state(&self.depth_stencil_state);

        render_encoder.set_fragment_bytes(
            BufferIndexLights as u64,
            std::mem::size_of::<Light>() as u64 * self.lighting.count as u64,
            self.lighting.lights.as_ptr() as *const _,
        );

        for model in self.models.iter() {
            self.uniforms[0].modelMatrix = unsafe { std::mem::transmute(model.model_matrix()) };
            self.uniforms[0].normalMatrix =
                unsafe { std::mem::transmute(Mat3A::from_mat4(model.model_matrix())) };
            render_encoder.set_vertex_bytes(
                BufferIndexUniforms as u64,
                std::mem::size_of::<Uniforms>() as u64,
                self.uniforms.as_ptr() as *const _,
            );

            self.fragment_uniforms[0].tiling = model.tiling;
            render_encoder.set_fragment_bytes(
                BufferIndexFragmentUniforms as u64,
                std::mem::size_of::<FragementUniforms>() as u64,
                self.fragment_uniforms.as_ptr() as *const _,
            );

            render_encoder.set_fragment_sampler_state(0, Some(&model.sampler_state));

            for submesh in model.submeshes.iter() {
                render_encoder.set_render_pipeline_state(&submesh.pipeline_state);

                render_encoder.set_vertex_buffer(
                    BufferIndexVertices as u64,
                    Some(&submesh.vertex_buffer),
                    0,
                );

                if let Some(textures) = &submesh.textures {
                    if let Some(diffuse_texture) = &textures.diffuse_texture {
                        render_encoder.set_fragment_texture(
                            Textures_BaseColorTexture as u64,
                            Some(&diffuse_texture),
                        );
                    }

                    if let Some(normal_texture) = &textures.normal_texture {
                        render_encoder.set_fragment_texture(
                            Textures_NormalTexture as u64,
                            Some(&normal_texture),
                        );
                    }
                }

                // render_encoder.set_triangle_fill_mode(MTLTriangleFillMode::Lines);
                render_encoder.draw_indexed_primitives(
                    MTLPrimitiveType::Triangle,
                    submesh.num_elements,
                    MTLIndexType::UInt32,
                    &submesh.index_buffer,
                    0,
                );
            }
        }

        render_encoder.end_encoding();

        command_buffer.present_drawable(&drawable);
        command_buffer.commit();
    }

    fn build_depth_stencil_state(device: &Device) -> DepthStencilState {
        let descriptor = DepthStencilDescriptor::new();
        descriptor.set_depth_compare_function(MTLCompareFunction::Less);
        descriptor.set_depth_write_enabled(true);
        device.new_depth_stencil_state(&descriptor)
    }
}
