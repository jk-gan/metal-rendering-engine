use crate::model::{Model, Submesh};
use crate::shader_bindings::{
    Attributes_Bitangent, Attributes_Normal, Attributes_Position, Attributes_Tangent,
    Attributes_UV, BufferIndices_BufferIndexSkybox as BufferIndexSkybox, Textures_BRDFLut,
    Textures_CubeMap, Textures_CubeMapDiffuse, Uniforms,
};
use glam::{Mat4, Vec4};
use image;
use image::{error::ImageResult, GenericImageView};
use metal::*;
use std::fs::File;
use std::io::BufReader;
use std::mem;
use std::path::Path;

pub struct Skybox {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_elements: u64,
    cube_map: Option<Texture>,
    irradiance_map: Option<Texture>,
    brdf_lut: Option<Texture>,
    pipeline_state: RenderPipelineState,
    depth_stencil_state: DepthStencilState,
}

impl Skybox {
    pub fn new(
        library: &Library,
        environment_library: &Library,
        device: &Device,
        brdf_lut: Option<Texture>,
    ) -> Self {
        let model = Model::from_gltf_filename("cube.gltf", 1, device, library);
        let pipeline_state = Self::build_pipeline_state(environment_library, device);
        let depth_stencil_state = Self::build_depth_stencil_state(device);
        let cube_map = Self::load_cube_map(device).ok();
        let irradiance_map = Self::load_irradiance_map(device).ok();

        let Submesh {
            vertex_buffer,
            index_buffer,
            num_elements,
            ..
        } = &model.meshes[0].submeshes[0];

        Self {
            vertex_buffer: vertex_buffer.clone(),
            index_buffer: index_buffer.clone(),
            num_elements: num_elements.clone(),
            cube_map,
            irradiance_map,
            brdf_lut,
            pipeline_state,
            depth_stencil_state,
        }
    }

    pub fn render(
        &self,
        render_encoder: &RenderCommandEncoderRef,
        uniforms: &mut [Uniforms],
        // fragment_uniforms: &mut [FragmentUniforms],
    ) {
        render_encoder.push_debug_group("Skybox");
        render_encoder.set_render_pipeline_state(&self.pipeline_state);
        render_encoder.set_depth_stencil_state(&self.depth_stencil_state);
        render_encoder.set_vertex_buffer(BufferIndexSkybox as u64, Some(&self.vertex_buffer), 0);

        let mut view_matrix: Mat4 = unsafe { mem::transmute(uniforms[0].viewMatrix) };
        let col3 = view_matrix.col_mut(3);
        *col3 = Vec4::new(0.0, 0.0, 0.0, 1.0);
        let mut projection_matrix: Mat4 = unsafe { mem::transmute(uniforms[0].projectionMatrix) };
        projection_matrix = projection_matrix * view_matrix;

        render_encoder.set_vertex_bytes(
            1,
            std::mem::size_of::<Mat4>() as u64,
            [projection_matrix].as_ptr() as *const _,
        );

        render_encoder.set_fragment_bytes(
            1,
            std::mem::size_of::<Uniforms>() as u64,
            uniforms.as_ptr() as *const _,
        );

        render_encoder.set_fragment_texture(
            Textures_CubeMap as u64,
            Some(&self.cube_map.as_ref().unwrap()),
        );

        render_encoder.draw_indexed_primitives(
            MTLPrimitiveType::Triangle,
            self.num_elements,
            MTLIndexType::UInt32,
            &self.index_buffer,
            0,
        );
    }

    pub fn update(&self, render_encoder: &RenderCommandEncoderRef) {
        if let Some(cube_map) = &self.cube_map {
            render_encoder.set_fragment_texture(Textures_CubeMap as u64, Some(&cube_map));
        }

        if let Some(irradiance_map) = &self.irradiance_map {
            render_encoder
                .set_fragment_texture(Textures_CubeMapDiffuse as u64, Some(&irradiance_map));
        }

        if let Some(brdf_lut) = &self.brdf_lut {
            render_encoder.set_fragment_texture(Textures_BRDFLut as u64, Some(&brdf_lut));
        }
    }

    fn build_pipeline_state(library: &Library, device: &Device) -> RenderPipelineState {
        let vertex_function = library.get_function("vertex_skybox", None).unwrap();
        let fragment_function = library.get_function("fragment_skybox", None).unwrap();

        let vertex_descriptor = VertexDescriptor::new();
        let mut offset = 0;

        // position
        let attribute_0 = vertex_descriptor
            .attributes()
            .object_at(Attributes_Position as u64)
            .unwrap();
        attribute_0.set_format(MTLVertexFormat::Float3);
        attribute_0.set_offset(offset);
        attribute_0.set_buffer_index(BufferIndexSkybox as u64);

        offset += mem::size_of::<f32>() as u64 * 3;

        // normal
        let attribute_1 = vertex_descriptor
            .attributes()
            .object_at(Attributes_Normal as u64)
            .unwrap();
        attribute_1.set_format(MTLVertexFormat::Float3);
        attribute_1.set_offset(offset);
        attribute_1.set_buffer_index(BufferIndexSkybox as u64);

        offset += mem::size_of::<f32>() as u64 * 3;

        // UV
        let attribute_2 = vertex_descriptor
            .attributes()
            .object_at(Attributes_UV as u64)
            .unwrap();
        attribute_2.set_format(MTLVertexFormat::Float2);
        attribute_2.set_offset(offset);
        attribute_2.set_buffer_index(BufferIndexSkybox as u64);

        offset += mem::size_of::<f32>() as u64 * 2;

        // tangent
        let attribute_3 = vertex_descriptor
            .attributes()
            .object_at(Attributes_Tangent as u64)
            .unwrap();
        attribute_3.set_format(MTLVertexFormat::Float3);
        attribute_3.set_offset(offset);
        attribute_3.set_buffer_index(BufferIndexSkybox as u64);

        offset += mem::size_of::<f32>() as u64 * 3;

        // bitangent
        let attribute_4 = vertex_descriptor
            .attributes()
            .object_at(Attributes_Bitangent as u64)
            .unwrap();
        attribute_4.set_format(MTLVertexFormat::Float3);
        attribute_4.set_offset(offset);
        attribute_4.set_buffer_index(BufferIndexSkybox as u64);

        offset += mem::size_of::<f32>() as u64 * 3;

        let layout_0 = vertex_descriptor
            .layouts()
            .object_at(BufferIndexSkybox as u64)
            .unwrap();
        layout_0.set_stride(offset);

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

    fn build_depth_stencil_state(device: &Device) -> DepthStencilState {
        let descriptor = DepthStencilDescriptor::new();
        descriptor.set_depth_compare_function(MTLCompareFunction::LessEqual);
        descriptor.set_depth_write_enabled(true);
        device.new_depth_stencil_state(&descriptor)
    }

    fn load_cube_map(device: &Device) -> ImageResult<Texture> {
        println!("Load cube map");
        let texture_descriptor = TextureDescriptor::new();
        texture_descriptor.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        texture_descriptor.set_texture_type(MTLTextureType::Cube);
        // texture_descriptor.set_width(width as u64);
        // texture_descriptor.set_height(height as u64);
        texture_descriptor.set_width(2048);
        texture_descriptor.set_height(2048);
        texture_descriptor.set_mipmap_level_count(9);
        // texture_descriptor.set_mipmap_level_count_for_size(MTLSize {
        //     width: width as u64,
        //     height: height as u64,
        //     depth: 1,
        // });
        let texture = device.new_texture(&texture_descriptor);

        // Load HDR equirectangular texture
        // let hdr_file =
        //     File::open("assets/environments/venice_sunset/venice_sunset_4k.hdr").unwrap();
        // let hdr_file = BufReader::new(hdr_file);
        // let hdr_decoder = image::hdr::HdrDecoder::new(hdr_file).unwrap();
        // let hdr_metadata = hdr_decoder.metadata();

        // let hdr_pixels = hdr_decoder.read_image_hdr().unwrap();

        // let mut hdr_pixels_raw = Vec::new();
        // for pixel in hdr_pixels {
        //     hdr_pixels_raw.push(pixel[0]);
        //     hdr_pixels_raw.push(pixel[1]);
        //     hdr_pixels_raw.push(pixel[2]);
        //     hdr_pixels_raw.push(1.0);
        // }

        let cubemaps = ["right", "left", "top", "bottom", "front", "back"];
        for (i, map) in cubemaps.iter().enumerate() {
            let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(format!("assets/environments/sea/{}.jpg", map));

            let img = image::open(path).unwrap();
            let (width, height) = img.dimensions();
            println!("dimensions: {}x{}", width, height);

            let region = MTLRegion::new_2d(0, 0, width as u64, height as u64);

            let image_scale = 1;
            let cube_size = width * image_scale;
            let bytes_per_pixel = 4;
            let bytes_per_row = bytes_per_pixel * cube_size;
            let bytes_per_image = bytes_per_row * cube_size;

            let mut new_buf: Vec<u8> = vec![];

            match img {
                image::DynamicImage::ImageRgb8(img) => {
                    println!("Image RGB 8");
                    for pixel in img.pixels() {
                        new_buf.push(pixel[2]);
                        new_buf.push(pixel[1]);
                        new_buf.push(pixel[0]);
                        // new_buf.push(pixel[0]);
                        // new_buf.push(pixel[1]);
                        // new_buf.push(pixel[2]);
                        new_buf.push(255);
                    }
                }
                image::DynamicImage::ImageRgba8(img) => {
                    println!("Image RGBA 8");
                    for pixel in img.pixels() {
                        new_buf.push(pixel[2]);
                        new_buf.push(pixel[1]);
                        new_buf.push(pixel[0]);
                        // new_buf.push(pixel[0]);
                        // new_buf.push(pixel[1]);
                        // new_buf.push(pixel[2]);
                        new_buf.push(pixel[3]);
                    }
                }
                _ => {
                    todo!()
                }
            }

            texture.replace_region_in_slice(
                region,
                0,
                i as u64,
                new_buf.as_ptr() as _,
                bytes_per_row as u64,
                bytes_per_image as u64,
            );

            for mipmap_level in 1..=8 {
                let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
                    "assets/environments/sea/specular/{}/specular-{}-{}.png",
                    mipmap_level, mipmap_level, map
                ));

                let img = image::open(path)?;
                let (width, height) = img.dimensions();
                println!("dimensions: {}x{}", width, height);

                let region = MTLRegion::new_2d(0, 0, width as u64, height as u64);

                let image_scale = 1;
                let cube_size = width * image_scale;
                let bytes_per_pixel = 4;
                let bytes_per_row = bytes_per_pixel * cube_size;
                let bytes_per_image = bytes_per_row * cube_size;

                let mut new_buf: Vec<u8> = vec![];

                match img {
                    image::DynamicImage::ImageRgb8(img) => {
                        for pixel in img.pixels() {
                            new_buf.push(pixel[2]);
                            new_buf.push(pixel[1]);
                            new_buf.push(pixel[0]);
                            new_buf.push(255);
                        }
                    }
                    image::DynamicImage::ImageRgba8(img) => {
                        for pixel in img.pixels() {
                            new_buf.push(pixel[2]);
                            new_buf.push(pixel[1]);
                            new_buf.push(pixel[0]);
                            new_buf.push(pixel[3]);
                        }
                    }
                    _ => {
                        todo!()
                    }
                }

                texture.replace_region_in_slice(
                    region,
                    mipmap_level as u64,
                    i as u64,
                    new_buf.as_ptr() as _,
                    bytes_per_row as u64,
                    bytes_per_image as u64,
                );
            }
        }
        Ok(texture)
    }

    fn load_irradiance_map(device: &Device) -> ImageResult<Texture> {
        let each_size = 128;
        let texture_descriptor = TextureDescriptor::new();
        texture_descriptor.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        texture_descriptor.set_texture_type(MTLTextureType::Cube);
        texture_descriptor.set_width(each_size);
        texture_descriptor.set_height(each_size);

        let texture = device.new_texture(&texture_descriptor);

        let map = "irradiance-6.png";
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("assets/environments/venice_sunset/{}", map));

        let img = image::open(path)?;
        let (width, height) = img.dimensions();
        println!("dimensions: {}x{}", width, height);

        for i in 0..6 {
            let start = i * each_size;
            let end = (i * each_size) + (each_size - 1);
            let region = MTLRegion::new_2d(0, 0, each_size, each_size);

            let image_scale = 1;
            let cube_size = each_size * image_scale;
            let bytes_per_pixel = 4;
            let bytes_per_row = bytes_per_pixel * cube_size;
            let bytes_per_image = bytes_per_row * cube_size;

            let mut new_buf: Vec<u8> = vec![];

            match img {
                image::DynamicImage::ImageRgb8(ref img) => {
                    for i in start..=end {
                        let row = img.rows().nth(i as usize).unwrap();
                        for pixel in row {
                            new_buf.push(pixel[2]);
                            new_buf.push(pixel[1]);
                            new_buf.push(pixel[0]);
                            new_buf.push(255);
                        }
                    }
                }
                image::DynamicImage::ImageRgba8(ref img) => {
                    for i in start..=end {
                        let row = img.rows().nth(i as usize).unwrap();
                        for pixel in row {
                            new_buf.push(pixel[2]);
                            new_buf.push(pixel[1]);
                            new_buf.push(pixel[0]);
                            new_buf.push(pixel[3]);
                        }
                    }
                }
                _ => {
                    todo!()
                }
            }

            texture.replace_region_in_slice(
                region,
                0,
                i as u64,
                new_buf.as_ptr() as _,
                bytes_per_row as u64,
                bytes_per_image as u64,
            );
        }
        Ok(texture)
    }

    // fn load_hdr_map(device: &Device) -> ImageResult<Texture> {
    //     let texture_descriptor = TextureDescriptor::new();
    //     texture_descriptor.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    //     texture_descriptor.set_texture_type(MTLTextureType::Cube);
    //     // texture_descriptor.set_width(width as u64);
    //     // texture_descriptor.set_height(height as u64);
    //     texture_descriptor.set_width(2048);
    //     texture_descriptor.set_height(2048);
    //     // texture_descriptor.set_mipmap_level_count_for_size(MTLSize {
    //     //     width: width as u64,
    //     //     height: height as u64,
    //     //     depth: 1,
    //     // });
    //     let texture = device.new_texture(&texture_descriptor);

    //     let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //         .join(format!("assets/environments/{}", "venice_sunset_4k.hdr"));

    //     let img = image::open(path)?;

    //     let (width, height) = img.dimensions();
    //     println!("dimensions: {}x{}", width, height);

    //     let region = MTLRegion::new_2d(0, 0, width as u64, height as u64);

    //     let image_scale = 1;
    //     let cube_size = width * image_scale;
    //     let bytes_per_pixel = 4;
    //     let bytes_per_row = bytes_per_pixel * cube_size;
    //     let bytes_per_image = bytes_per_row * cube_size;

    //     let mut new_buf: Vec<u8> = vec![];

    //     match img {
    //         image::DynamicImage::ImageRgb8(img) => {
    //             for pixel in img.pixels() {
    //                 new_buf.push(pixel[2]);
    //                 new_buf.push(pixel[1]);
    //                 new_buf.push(pixel[0]);
    //                 new_buf.push(255);
    //             }
    //         }
    //         image::DynamicImage::ImageRgba8(img) => {
    //             for pixel in img.pixels() {
    //                 new_buf.push(pixel[2]);
    //                 new_buf.push(pixel[1]);
    //                 new_buf.push(pixel[0]);
    //                 new_buf.push(pixel[3]);
    //             }
    //         }
    //         _ => {
    //             todo!()
    //         }
    //     }

    //     texture.replace_region_in_slice(
    //         region,
    //         0,
    //         i as u64,
    //         new_buf.as_ptr() as _,
    //         bytes_per_row as u64,
    //         bytes_per_image as u64,
    //     );
    //     Ok(texture)
    // }
}
