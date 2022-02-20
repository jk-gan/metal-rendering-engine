use crate::camera::{ArcballCamera, CameraFunction};
use crate::shader_bindings::{
    BufferIndices_BufferIndexLights as BufferIndexLights, FragmentUniforms, Light, Uniforms,
};
use crate::{lighting::Lighting, model::Model, skybox::Skybox};
use cocoa::{appkit::NSView, base::id as cocoa_id};
use core_graphics_types::geometry::CGSize;
use glam::{Mat3A, Mat4, Vec3, Vec3A};
use metal::*;
use objc::runtime::YES;
use winit::{platform::macos::WindowExtMacOS, window::Window};

pub struct Renderer {
    draw_size_width: u64,
    draw_size_height: u64,
    layer: MetalLayer,
    pub device: Device,
    command_queue: CommandQueue,
    library: Library,
    uniforms: [Uniforms; 1],
    skybox_uniforms: [Uniforms; 1],
    fragment_uniforms: [FragmentUniforms; 1],
    camera: ArcballCamera,
    models: Vec<Model>,
    skybox: Option<Skybox>,
    depth_stencil_state: DepthStencilState,
    lighting: Lighting,
}

fn get_high_performance_device() -> Option<Device> {
    let devices_list = Device::all();
    for device in devices_list {
        if !device.is_low_power() && !device.is_removable() && !device.is_headless() {
            return Some(device);
        }
    }

    None
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        #[cfg(not(target_os = "macos"))]
        panic!("The viewer only support macOS at the moment.");

        let device = get_high_performance_device().expect("Discrete GPU not available!");
        println!("support raytracing? {}", device.supports_raytracing());
        println!(
            "support Apple family 6? {}",
            device.supports_family(MTLGPUFamily::Apple6)
        );
        println!(
            "support Apple Mac 7? {}",
            device.supports_family(MTLGPUFamily::Mac2)
        );

        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_presents_with_transaction(false);

        unsafe {
            let view = window.ns_view() as cocoa_id;
            view.setWantsLayer(YES);
            view.setLayer(std::mem::transmute(layer.as_ref()));
        }

        let draw_size = window.inner_size();
        layer.set_drawable_size(CGSize::new(draw_size.width as f64, draw_size.height as f64));

        let command_queue = device.new_command_queue();

        let library_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/shaders/pbr.metallib");
        let library = device.new_library_with_file(library_path).unwrap();

        let brdf_lut = Self::build_brdf(&device, &library, &command_queue);

        let skybox = Skybox::new(&library, &device, brdf_lut);

        let (camera, model) = Self::read_gltf_asset(&device, &library);

        let models = vec![model];

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

        let skybox_uniforms = Uniforms {
            modelMatrix: unsafe { std::mem::transmute(Mat4::IDENTITY) },
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
            layer,
            library,
            draw_size_width: draw_size.width as u64,
            draw_size_height: draw_size.height as u64,
            uniforms: [uniforms],
            skybox_uniforms: [skybox_uniforms],
            fragment_uniforms: [fragment_uniforms],
            camera,
            models,
            skybox: Some(skybox),
            depth_stencil_state,
            lighting,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.layer
            .set_drawable_size(CGSize::new(width as f64, height as f64));

        let aspect_ratio = width as f32 / height as f32;
        self.camera.set_aspect_ratio(aspect_ratio);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.camera.zoom(delta);
    }

    pub fn rotate(&mut self, delta: (f64, f64)) {
        self.camera.rotate((delta.0 as f32, delta.1 as f32));
    }

    pub fn draw(&mut self) {
        let drawable = match self.layer.next_drawable() {
            Some(drawable) => drawable,
            None => return,
        };

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
        depth_buffer_descriptor.set_width(self.draw_size_width);
        depth_buffer_descriptor.set_height(self.draw_size_height);
        depth_buffer_descriptor.set_pixel_format(MTLPixelFormat::Depth32Float);
        depth_buffer_descriptor.set_storage_mode(MTLStorageMode::Private);
        depth_buffer_descriptor
            .set_usage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
        let depth_attachment = render_pass_descriptor.depth_attachment().unwrap();
        depth_attachment.set_texture(Some(&self.device.new_texture(&depth_buffer_descriptor)));
        depth_attachment.set_load_action(MTLLoadAction::Clear);
        depth_attachment.set_store_action(MTLStoreAction::DontCare);
        depth_attachment.set_clear_depth(1.0);

        self.uniforms[0].projectionMatrix =
            unsafe { std::mem::transmute(*self.camera.projection_matrix()) };
        self.uniforms[0].viewMatrix = unsafe { std::mem::transmute(*self.camera.view_matrix()) };

        self.skybox_uniforms[0].viewMatrix = self.uniforms[0].viewMatrix;
        self.skybox_uniforms[0].projectionMatrix = self.uniforms[0].projectionMatrix;

        self.fragment_uniforms[0].cameraPosition = unsafe {
            std::mem::transmute(Vec3A::new(
                self.camera.position().x,
                self.camera.position().y,
                self.camera.position().z,
            ))
        };

        let command_buffer = self.command_queue.new_command_buffer();
        let render_encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);
        // render_encoder.set_front_facing_winding(MTLWinding::CounterClockwise);
        // render_encoder.set_cull_mode(MTLCullMode::Back);
        render_encoder.set_depth_stencil_state(&self.depth_stencil_state);

        render_encoder.set_fragment_bytes(
            BufferIndexLights as u64,
            std::mem::size_of::<Light>() as u64 * self.lighting.count as u64,
            self.lighting.lights.as_ptr() as *const _,
        );

        if let Some(skybox) = &self.skybox {
            skybox.update(&render_encoder);
        }

        for model in self.models.iter() {
            render_encoder.push_debug_group(&model.name());
            model.render(
                &render_encoder,
                &mut self.uniforms,
                &mut self.fragment_uniforms,
            );
            render_encoder.pop_debug_group();
        }

        if let Some(skybox) = &self.skybox {
            skybox.render(&render_encoder, &mut self.skybox_uniforms);
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
        let mut model =
            Model::from_gltf_filename("DamagedHelmet/DamagedHelmet.gltf", 1, &device, &library);
        model.set_rotation(Vec3::new(
            270.0_f32.to_radians(),
            180.0_f32.to_radians(),
            220.0_f32.to_radians(),
        ));
        // let model =
        //     Model::from_gltf_filename("FlightHelmet/FlightHelmet.gltf", 1, &device, &library);

        let mut camera = ArcballCamera::new(0.5, 10.0, Vec3::new(0.0, 0.3, 0.0), 3.5);
        camera.set_position(Vec3::new(0.0, 0.0, 2.5));
        camera.set_rotation(Vec3::new(0.0, 160.0_f32.to_radians(), 0.0));

        (camera, model)
    }

    fn build_brdf(
        device: &Device,
        library: &Library,
        command_queue: &CommandQueue,
    ) -> Option<Texture> {
        let size = 256;

        let brdf_function = library.get_function("integrateBRDF", None).unwrap();
        let brdf_pipeline_state =
            match device.new_compute_pipeline_state_with_function(&brdf_function) {
                Ok(pipeline_state) => pipeline_state,
                _ => return None,
            };
        let command_buffer = command_queue.new_command_buffer();
        let command_encoder = command_buffer.new_compute_command_encoder();

        let descriptor = TextureDescriptor::new();
        descriptor.set_width(size);
        descriptor.set_height(size);
        descriptor.set_pixel_format(MTLPixelFormat::RG16Float);
        // descriptor.set_mipmap_level_count(0);
        descriptor.set_usage(MTLTextureUsage::ShaderWrite | MTLTextureUsage::ShaderRead);
        let lut = device.new_texture(&descriptor);
        command_encoder.set_compute_pipeline_state(&brdf_pipeline_state);
        command_encoder.set_texture(0, Some(&lut));
        let threads_per_threadgroup = MTLSize::new(16, 16, 1);
        let threadgroups = MTLSize::new(
            size / threads_per_threadgroup.width,
            size / threads_per_threadgroup.height,
            1,
        );
        command_encoder.dispatch_thread_groups(threadgroups, threads_per_threadgroup);
        command_encoder.end_encoding();
        command_buffer.commit();

        Some(lut)
    }
}
