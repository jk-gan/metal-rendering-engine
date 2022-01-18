use crate::camera::{ArcballCamera, CameraFunction};
use crate::shader_bindings::{
    BufferIndices_BufferIndexLights as BufferIndexLights, FragmentUniforms, Light, Uniforms,
};
use crate::{lighting::Lighting, model::Model};
use glam::{Mat3A, Mat4, Vec3, Vec3A};
use metal::*;

pub struct Renderer {
    pub device: Device,
    command_queue: CommandQueue,
    library: Library,
    uniforms: [Uniforms; 1],
    fragment_uniforms: [FragmentUniforms; 1],
    camera: ArcballCamera,
    models: Vec<Model>,
    depth_stencil_state: DepthStencilState,
    lighting: Lighting,
}

impl Renderer {
    pub fn new() -> Self {
        let device = Device::system_default().expect("GPU not available!");
        let command_queue = device.new_command_queue();

        // let library_path =
        //     std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders/shaders.metallib");
        let library_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/shaders/pbr.metallib");
        let library = device.new_library_with_file(library_path).unwrap();

        let (camera, damaged_helmet) = Self::read_gltf_asset(&device, &library);

        let models = vec![damaged_helmet];

        // generate mipmaps
        for model in models.iter() {
            for mesh in model.meshes.iter() {
                for submesh in mesh.submeshes.iter() {
                    let command_buffer = command_queue.new_command_buffer();
                    let blit_command_encoder = command_buffer.new_blit_command_encoder();

                    if let Some(texture) = &submesh.textures.diffuse_texture {
                        blit_command_encoder.generate_mipmaps(&texture);
                    }
                    if let Some(texture) = &submesh.textures.normal_texture {
                        blit_command_encoder.generate_mipmaps(&texture);
                    }
                    if let Some(texture) = &submesh.textures.metallic_roughness_texture {
                        blit_command_encoder.generate_mipmaps(&texture);
                    }
                    if let Some(texture) = &submesh.textures.emissive_texture {
                        blit_command_encoder.generate_mipmaps(&texture);
                    }
                    if let Some(texture) = &submesh.textures.ambient_occlusion_texture {
                        blit_command_encoder.generate_mipmaps(&texture);
                    }

                    blit_command_encoder.end_encoding();
                    command_buffer.commit();
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

        let fragment_uniforms = FragmentUniforms {
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
        color_attachment.set_clear_color(MTLClearColor::new(0.2, 0.2, 0.25, 1.0));
        // color_attachment.set_clear_color(MTLClearColor::new(0.93, 0.97, 1.0, 1.0));
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
            render_encoder.push_debug_group(&model.name());
            model.render(
                &render_encoder,
                &mut self.uniforms,
                &mut self.fragment_uniforms,
            );
            render_encoder.pop_debug_group();
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

    fn read_gltf_asset(device: &Device, library: &Library) -> (ArcballCamera, Model) {
        // let damaged_helmet =
        //     Model::from_gltf_filename("DamagedHelmet/DamagedHelmet.gltf", 1, &device, &library);
        let model =
            Model::from_gltf_filename("FlightHelmet/FlightHelmet.gltf", 1, &device, &library);

        let mut camera = ArcballCamera::new(0.5, 10.0, Vec3::new(0.0, 0.3, 0.0), 1.0);
        camera.set_position(Vec3::new(0.0, 0.0, 2.5));
        camera.set_rotation(Vec3::new(0.0, 160.0_f32.to_radians(), 0.0));

        (camera, model)
    }
}
