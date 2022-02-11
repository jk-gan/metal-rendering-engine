use crate::shader_bindings::{
    Attributes_Bitangent, Attributes_Normal, Attributes_Position, Attributes_Tangent,
    Attributes_UV, BufferIndices_BufferIndexFragmentUniforms as BufferIndexFragmentUniforms,
    BufferIndices_BufferIndexMaterials as BufferIndexMaterials,
    BufferIndices_BufferIndexUniforms as BufferIndexUniforms,
    BufferIndices_BufferIndexVertices as BufferIndexVertices, FragmentUniforms, Material,
    Textures_BaseColorTexture, Textures_EmissiveTexture, Textures_MetallicRoughnessTexture,
    Textures_NormalTexture, Textures_OcclusionTexture, Uniforms,
};
use crate::{node::InnerNode, texturable::Texturable};
use glam::{f32::Quat, Mat3A, Mat4, Vec2, Vec3, Vec4};
use metal::*;
use std::mem;

const TEXTURE_PATH: &str = "DamagedHelmet";
// const TEXTURE_PATH: &str = "FlightHelmet";

#[derive(Debug, Copy, Clone)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub text_coords: [f32; 2],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl Default for ModelVertex {
    fn default() -> Self {
        ModelVertex {
            position: [0.0; 3],
            normal: [0.0; 3],
            text_coords: [0.0; 2],
            tangent: [0.0; 3],
            bitangent: [0.0; 3],
        }
    }
}

impl Material {
    pub fn new(
        base_color: [f32; 4],
        specular_color: [f32; 4],
        shininess: f32,
        roughness: f32,
        metallic: f32,
    ) -> Self {
        unsafe {
            Self {
                baseColor: std::mem::transmute(Vec4::from(base_color)),
                specularColor: std::mem::transmute(Vec4::from(specular_color)),
                shininess: std::mem::transmute(shininess),
                roughness: std::mem::transmute(roughness),
                metallic: std::mem::transmute(metallic),
            }
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new([1.0, 1.0, 1.0, 1.0], [1.0, 1.0, 1.0, 1.0], 32.0, 0.0, 0.0)
    }
}

pub struct Submesh {
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
    pub(crate) num_elements: u64,
    pub(crate) textures: Textures,
    pub(crate) pipeline_state: RenderPipelineState,
    pub(crate) material: [Material; 1],
}

impl Submesh {
    pub fn from_gltf(
        device: &Device,
        library: &Library,
        material: &Option<gltf::Material>,
        vertex_buffer: Buffer,
        index_buffer: Buffer,
        num_elements: u64,
    ) -> Self {
        let mut textures = Textures::default();
        let mut pbr_material = Material::default();

        if let Some(material) = material {
            let normal_texture_source = material.normal_texture().map(|info| {
                println!("normal text_coord index: {}", info.tex_coord());
                match info.texture().source().source() {
                    gltf::image::Source::Uri { uri, .. } => uri,
                    x => {
                        println!("x = {:?}", x);
                        todo!()
                    }
                }
            });

            let normal_texture = normal_texture_source.map(|source| {
                Self::load_texture(format!("{}/{}", TEXTURE_PATH, source).as_ref(), &device)
                    .expect("Unable to load normal texture")
            });

            let occlusion_texture_source = material.occlusion_texture().map(|info| {
                println!("occlusion text_coord index: {}", info.tex_coord());
                match info.texture().source().source() {
                    gltf::image::Source::Uri { uri, .. } => uri,
                    x => {
                        println!("x = {:?}", x);
                        todo!()
                    }
                }
            });

            let occlusion_texture = occlusion_texture_source.map(|source| {
                Self::load_texture(format!("{}/{}", TEXTURE_PATH, source).as_ref(), &device)
                    .expect("Unable to load occlusion texture")
            });

            let emissive_texture_source = material.emissive_texture().map(|info| {
                println!("emissive text_coord index: {}", info.tex_coord());
                match info.texture().source().source() {
                    gltf::image::Source::Uri { uri, .. } => uri,
                    x => {
                        println!("x = {:?}", x);
                        todo!()
                    }
                }
            });

            let emissive_texture = emissive_texture_source.map(|source| {
                Self::load_texture(format!("{}/{}", TEXTURE_PATH, source).as_ref(), &device)
                    .expect("Unable to load emissive texture")
            });

            let emissive_factor = material.emissive_factor();
            println!("emissive factor: {:?}", emissive_factor);

            let pbr_metallic_roughness = material.pbr_metallic_roughness();
            let base_color_factor = pbr_metallic_roughness.base_color_factor();
            let metallic_factor = pbr_metallic_roughness.metallic_factor();
            let roughness_factor = pbr_metallic_roughness.roughness_factor();

            let base_color_texture_source =
                pbr_metallic_roughness.base_color_texture().map(|info| {
                    println!("base color text_coord index: {}", info.tex_coord());
                    match info.texture().source().source() {
                        gltf::image::Source::Uri { uri, .. } => uri,
                        x => {
                            println!("x = {:?}", x);
                            todo!()
                        }
                    }
                });

            let base_color_texture = base_color_texture_source.map(|source| {
                Self::load_texture(format!("{}/{}", TEXTURE_PATH, source).as_ref(), &device)
                    .expect("Unable to load base color texture")
            });

            let metallic_roughness_texture_source = pbr_metallic_roughness
                .metallic_roughness_texture()
                .map(|info| {
                    println!("metallic roughness text_coord index: {}", info.tex_coord());
                    match info.texture().source().source() {
                        gltf::image::Source::Uri { uri, .. } => uri,
                        x => {
                            println!("x = {:?}", x);
                            todo!()
                        }
                    }
                });

            let metallic_roughness_texture = metallic_roughness_texture_source.map(|source| {
                Self::load_texture(format!("{}/{}", TEXTURE_PATH, source).as_ref(), &device)
                    .expect("Unable to load metallic roughness texture")
            });

            textures = Textures::new(
                base_color_texture,
                normal_texture,
                metallic_roughness_texture,
                occlusion_texture,
                emissive_texture,
            );
            pbr_material = Material::new(
                base_color_factor,
                [0.0, 0.0, 0.0, 0.0],
                0.0,
                roughness_factor,
                metallic_factor,
            );
        }

        let pipeline_state = Submesh::build_pipeline_state(library, device, &textures);

        Self {
            vertex_buffer,
            index_buffer,
            num_elements,
            textures,
            pipeline_state,
            material: [pbr_material],
        }
    }

    fn build_pipeline_state(
        library: &Library,
        device: &Device,
        textures: &Textures,
    ) -> RenderPipelineState {
        let fragment_constants = Self::make_function_constants(textures);

        let fragment_function = library
            .get_function("fragment_main", Some(fragment_constants))
            .expect("No Metal function exists");
        // let fragment_function = library
        //     .get_function("skybox_test", Some(fragment_constants))
        //     .expect("No Metal function exists");
        let vertex_function = library.get_function("vertex_main", None).unwrap();
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

    fn make_function_constants(textures: &Textures) -> FunctionConstantValues {
        let function_constants = FunctionConstantValues::new();
        function_constants.set_constant_value_at_index(
            [textures.diffuse_texture.is_some()].as_ptr() as *const _,
            MTLDataType::Bool,
            0,
        );
        function_constants.set_constant_value_at_index(
            [textures.normal_texture.is_some()].as_ptr() as *const _,
            MTLDataType::Bool,
            1,
        );
        // metallic roughness
        function_constants.set_constant_value_at_index(
            [textures.metallic_roughness_texture.is_some()].as_ptr() as *const _,
            MTLDataType::Bool,
            2,
        );
        // ambiemt occlusion
        function_constants.set_constant_value_at_index(
            [textures.ambient_occlusion_texture.is_some()].as_ptr() as *const _,
            MTLDataType::Bool,
            3,
        );
        // emissive
        function_constants.set_constant_value_at_index(
            [textures.emissive_texture.is_some()].as_ptr() as *const _,
            MTLDataType::Bool,
            4,
        );
        function_constants
    }
}

impl Texturable for Submesh {}

pub struct Mesh {
    name: String,
    inner_node: InnerNode,
    pub(crate) submeshes: Vec<Submesh>,
}

impl Mesh {
    fn from_gltf(
        device: &Device,
        library: &Library,
        buffers: &Vec<gltf::buffer::Data>,
        mesh: gltf::Mesh,
        material: Option<gltf::material::Material>,
    ) -> Mesh {
        let mut submeshes = vec![];
        for primitive in mesh.primitives() {
            println!("- Primitive #{}", primitive.index());
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let mut vertices = vec![];
            let mut indices = vec![];

            if let Some(iter) = reader.read_positions() {
                vertices = iter
                    .map(|vertex_position| ModelVertex {
                        position: [vertex_position[0], vertex_position[1], vertex_position[2]],
                        ..ModelVertex::default()
                    })
                    .collect()
            }

            if let Some(iter) = reader.read_indices() {
                indices = iter.into_u32().map(|index| index).collect()
            }

            if let Some(iter) = reader.read_normals() {
                for (i, vertex_normal) in iter.enumerate() {
                    vertices[i].normal = [vertex_normal[0], vertex_normal[1], vertex_normal[2]];
                }
            }

            if let Some(iter) = reader.read_tex_coords(0) {
                for (i, text_coord) in iter.into_f32().enumerate() {
                    vertices[i].text_coords = [text_coord[0], text_coord[1]];
                }
            }

            // if let Some(iter) = reader.read_tex_coords(1) {
            //     for (i, text_coord) in iter.into_f32().enumerate() {
            //         vertices[i].text_coords[1] = [text_coord[0], text_coord[1]];
            //     }
            // }

            if let Some(iter) = reader.read_tangents() {
                for (i, tangent) in iter.enumerate() {
                    vertices[i].tangent = [tangent[0], tangent[1], tangent[2]];
                    let normal_vector = Vec3::from(vertices[i].normal);
                    let tangent_vector = Vec3::from(vertices[i].tangent);
                    vertices[i].bitangent =
                        (normal_vector.cross(tangent_vector) * tangent[3]).into();
                }
            } else {
                let mut triangles_included = (0..vertices.len()).collect::<Vec<_>>();
                // Calculate tangents and bitangets. We're going to
                // use the triangles, so we need to loop through the
                // indices in chunks of 3
                for c in indices.chunks(3) {
                    let v0 = vertices[c[0] as usize];
                    let v1 = vertices[c[1] as usize];
                    let v2 = vertices[c[2] as usize];

                    let pos0: Vec3 = v0.position.into();
                    let pos1: Vec3 = v1.position.into();
                    let pos2: Vec3 = v2.position.into();

                    let uv0: Vec2 = v0.text_coords.into();
                    let uv1: Vec2 = v1.text_coords.into();
                    let uv2: Vec2 = v2.text_coords.into();

                    // Calculate the edges of the triangle
                    let delta_pos1 = pos1 - pos0;
                    let delta_pos2 = pos2 - pos0;

                    // This will give us a direction to calculate the
                    // tangent and bitangent
                    let delta_uv1 = uv1 - uv0;
                    let delta_uv2 = uv2 - uv0;

                    // Solving the following system of equations will
                    // give us the tangent and bitangent.
                    //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                    //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                    // Luckily, the place I found this equation provided
                    // the solution!
                    let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                    let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                    let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

                    // We'll use the same tangent/bitangent for each vertex in the triangle
                    vertices[c[0] as usize].tangent =
                        (tangent + Vec3::from(vertices[c[0] as usize].tangent)).into();
                    vertices[c[1] as usize].tangent =
                        (tangent + Vec3::from(vertices[c[1] as usize].tangent)).into();
                    vertices[c[2] as usize].tangent =
                        (tangent + Vec3::from(vertices[c[2] as usize].tangent)).into();
                    vertices[c[0] as usize].bitangent =
                        (bitangent + Vec3::from(vertices[c[0] as usize].bitangent)).into();
                    vertices[c[1] as usize].bitangent =
                        (bitangent + Vec3::from(vertices[c[1] as usize].bitangent)).into();
                    vertices[c[2] as usize].bitangent =
                        (bitangent + Vec3::from(vertices[c[2] as usize].bitangent)).into();

                    // Used to average the tangents/bitangents
                    triangles_included[c[0] as usize] += 1;
                    triangles_included[c[1] as usize] += 1;
                    triangles_included[c[2] as usize] += 1;
                }

                // Average the tangents/bitangents
                for (i, n) in triangles_included.into_iter().enumerate() {
                    let denom = 1.0 / n as f32;
                    let mut v = &mut vertices[i];
                    v.tangent = (Vec3::from(v.tangent) * denom).normalize().into();
                    v.bitangent = (Vec3::from(v.bitangent) * denom).normalize().into();
                }
            }

            let vertex_buffer = device.new_buffer_with_data(
                vertices.as_ptr() as *const _,
                mem::size_of::<ModelVertex>() as u64 * vertices.len() as u64,
                MTLResourceOptions::CPUCacheModeDefaultCache
                    | MTLResourceOptions::StorageModeManaged,
            );
            let index_buffer = device.new_buffer_with_data(
                indices.as_ptr() as *const _,
                mem::size_of::<u32>() as u64 * indices.len() as u64,
                MTLResourceOptions::CPUCacheModeDefaultCache
                    | MTLResourceOptions::StorageModeManaged,
            );
            let num_elements = indices.len() as u64;

            let submesh = Submesh::from_gltf(
                &device,
                &library,
                &material,
                vertex_buffer,
                index_buffer,
                num_elements,
            );
            submeshes.push(submesh);
        }
        let inner_node = InnerNode::default();

        Self {
            name: mesh.name().unwrap_or("untitled").to_string(),
            submeshes,
            inner_node,
        }
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.inner_node.position = position;
    }

    pub fn name(&self) -> &String {
        &self.inner_node.name
    }

    pub fn set_rotation(&mut self, rotation: Vec3) {
        self.inner_node.rotation = rotation;
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.inner_node.scale = scale;
    }

    pub fn apply_translation(&mut self, translation: Mat4) {
        todo!()
    }

    pub fn apply_rotation(&mut self, rotation: Mat4) {
        todo!()
    }

    pub fn apply_scale(&mut self, scale: Mat4) {
        todo!()
    }

    pub fn apply_transform_matrix(&mut self, transform_matrix: Mat4) {
        let current_position = self.inner_node.position;
        let current_rotation = self.inner_node.rotation;
        let current_scale = self.inner_node.scale;

        let (scale, rotation, translation) = transform_matrix.to_scale_rotation_translation();

        self.inner_node.position =
            (Mat4::from_translation(translation) * Vec4::from((current_position, 1.0))).truncate();
        self.inner_node.rotation = rotation.to_scaled_axis() + Vec3::from(current_rotation);
        self.inner_node.scale =
            (Mat4::from_scale(scale) * Vec4::from((current_scale, 1.0))).truncate();
    }

    pub fn model_matrix(&self) -> Mat4 {
        self.inner_node.model_matrix()
    }
}

pub struct Model {
    inner_node: InnerNode,
    pub(crate) meshes: Vec<Mesh>,
    pub(crate) tiling: u32,
    pub(crate) sampler_state: SamplerState,
}

impl Model {
    pub fn new(
        inner_node: InnerNode,
        meshes: Vec<Mesh>,
        tiling: u32,
        sampler_state: SamplerState,
    ) -> Model {
        Model {
            inner_node,
            meshes,
            tiling,
            sampler_state,
        }
    }

    pub fn from_gltf_filename(
        name: &str,
        tiling: u32,
        device: &Device,
        library: &Library,
    ) -> Model {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("assets/models/{}", name));
        let (gltf, buffers, _) = gltf::import(path.as_path()).expect("Failed to load gltf file");

        let mut meshes: Vec<Mesh> = vec![];

        println!("nodes len: {}", gltf.nodes().len());
        println!("cameras len: {}", gltf.cameras().len());
        println!("materials len: {}", gltf.materials().len());
        println!("meshes len: {}", gltf.meshes().len());

        for gltf_node in gltf.nodes() {
            println!("");
            println!("transform: {:?}", gltf_node.transform());
            println!("name: {:?}", gltf_node.name());
            println!("children len: {:?}", gltf_node.children().len());
            if let Some(gltf_mesh) = gltf_node.mesh() {
                println!("Mesh #{}", gltf_mesh.index());
                println!("name: {:?}", gltf_mesh.name());
                let material = gltf.materials().nth(gltf_mesh.index());

                let mesh = Mesh::from_gltf(device, library, &buffers, gltf_mesh, material);
                meshes.push(mesh);
            } else if let Some(gltf_camera) = gltf_node.camera() {
                println!("camera: {:?}", gltf_camera);
            }
        }

        let sampler_state = Model::build_sampler_state(device);

        let mut inner_node = InnerNode::default();
        inner_node.name = name.to_string();

        let mut model = Model::new(inner_node, meshes, tiling, sampler_state);

        let first_node = gltf.nodes().nth(0).unwrap();
        println!("transform: {:?}", first_node.transform());

        match first_node.transform() {
            gltf::scene::Transform::Matrix { matrix } => {
                model.apply_transform_matrix(Mat4::from_cols_array_2d(&matrix));
            }
            gltf::scene::Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => {
                let transform_matrix = Mat4::from_scale_rotation_translation(
                    Vec3::from(scale),
                    Quat::from_array(rotation),
                    Vec3::from(translation),
                );
                model.apply_transform_matrix(transform_matrix);
            }
        }

        model
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.inner_node.position = position;
    }

    pub fn name(&self) -> &String {
        &self.inner_node.name
    }

    pub fn set_rotation(&mut self, rotation: Vec3) {
        self.inner_node.rotation = rotation;
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.inner_node.scale = scale;
    }

    pub fn apply_translation(&mut self, translation: Mat4) {
        todo!()
    }

    pub fn apply_rotation(&mut self, rotation: Mat4) {
        todo!()
    }

    pub fn apply_scale(&mut self, scale: Mat4) {
        todo!()
    }

    pub fn apply_transform_matrix(&mut self, transform_matrix: Mat4) {
        let current_position = self.inner_node.position;
        let current_rotation = self.inner_node.rotation;
        let current_scale = self.inner_node.scale;

        let (scale, rotation, translation) = transform_matrix.to_scale_rotation_translation();

        self.inner_node.position =
            (Mat4::from_translation(translation) * Vec4::from((current_position, 1.0))).truncate();
        self.inner_node.rotation = rotation.to_scaled_axis() + Vec3::from(current_rotation);
        self.inner_node.scale =
            (Mat4::from_scale(scale) * Vec4::from((current_scale, 1.0))).truncate();
    }

    pub fn model_matrix(&self) -> Mat4 {
        self.inner_node.model_matrix()
    }

    pub fn render(
        &self,
        render_encoder: &RenderCommandEncoderRef,
        uniforms: &mut [Uniforms],
        fragment_uniforms: &mut [FragmentUniforms],
    ) {
        fragment_uniforms[0].tiling = self.tiling;
        render_encoder.set_fragment_bytes(
            BufferIndexFragmentUniforms as u64,
            std::mem::size_of::<FragmentUniforms>() as u64,
            fragment_uniforms.as_ptr() as *const _,
        );

        render_encoder.set_fragment_sampler_state(0, Some(&self.sampler_state));

        for mesh in self.meshes.iter() {
            uniforms[0].modelMatrix = unsafe { std::mem::transmute(self.model_matrix()) };
            uniforms[0].normalMatrix =
                unsafe { std::mem::transmute(Mat3A::from_mat4(self.model_matrix())) };
            render_encoder.set_vertex_bytes(
                BufferIndexUniforms as u64,
                std::mem::size_of::<Uniforms>() as u64,
                uniforms.as_ptr() as *const _,
            );

            for submesh in mesh.submeshes.iter() {
                render_encoder.set_render_pipeline_state(&submesh.pipeline_state);

                render_encoder.set_vertex_buffer(
                    BufferIndexVertices as u64,
                    Some(&submesh.vertex_buffer),
                    0,
                );

                if let Some(diffuse_texture) = &submesh.textures.diffuse_texture {
                    render_encoder.set_fragment_texture(
                        Textures_BaseColorTexture as u64,
                        Some(&diffuse_texture),
                    );
                }

                if let Some(normal_texture) = &submesh.textures.normal_texture {
                    render_encoder
                        .set_fragment_texture(Textures_NormalTexture as u64, Some(&normal_texture));
                }

                if let Some(metallic_roughness_texture) =
                    &submesh.textures.metallic_roughness_texture
                {
                    render_encoder.set_fragment_texture(
                        Textures_MetallicRoughnessTexture as u64,
                        Some(&metallic_roughness_texture),
                    );
                }

                if let Some(occlusion_texture) = &submesh.textures.ambient_occlusion_texture {
                    render_encoder.set_fragment_texture(
                        Textures_OcclusionTexture as u64,
                        Some(&occlusion_texture),
                    );
                }

                if let Some(emissive_texture) = &submesh.textures.emissive_texture {
                    render_encoder.set_fragment_texture(
                        Textures_EmissiveTexture as u64,
                        Some(&emissive_texture),
                    );
                }

                render_encoder.set_fragment_bytes(
                    BufferIndexMaterials as u64,
                    std::mem::size_of::<Material>() as u64,
                    submesh.material.as_ptr() as *const _,
                );

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
    }

    fn build_sampler_state(device: &Device) -> SamplerState {
        let descriptor = SamplerDescriptor::new();
        descriptor.set_address_mode_s(MTLSamplerAddressMode::Repeat);
        descriptor.set_address_mode_t(MTLSamplerAddressMode::Repeat);
        descriptor.set_mip_filter(MTLSamplerMipFilter::Nearest);
        // descriptor.set_mag_filter(MTLSamplerMinMagFilter::Nearest);
        // descriptor.set_min_filter(MTLSamplerMinMagFilter::Nearest);
        descriptor.set_max_anisotropy(8);
        device.new_sampler(&descriptor)
    }
}

fn default_vertex_descriptor() -> &'static VertexDescriptorRef {
    let vertex_descriptor = VertexDescriptor::new();
    let mut offset = 0;

    // position
    let attribute_0 = vertex_descriptor
        .attributes()
        .object_at(Attributes_Position as u64)
        .unwrap();
    attribute_0.set_format(MTLVertexFormat::Float3);
    attribute_0.set_offset(offset);
    attribute_0.set_buffer_index(BufferIndexVertices as u64);

    offset += mem::size_of::<f32>() as u64 * 3;

    // normal
    let attribute_1 = vertex_descriptor
        .attributes()
        .object_at(Attributes_Normal as u64)
        .unwrap();
    attribute_1.set_format(MTLVertexFormat::Float3);
    attribute_1.set_offset(offset);
    attribute_1.set_buffer_index(BufferIndexVertices as u64);

    offset += mem::size_of::<f32>() as u64 * 3;

    // UV
    let attribute_2 = vertex_descriptor
        .attributes()
        .object_at(Attributes_UV as u64)
        .unwrap();
    attribute_2.set_format(MTLVertexFormat::Float2);
    attribute_2.set_offset(offset);
    attribute_2.set_buffer_index(BufferIndexVertices as u64);

    offset += mem::size_of::<f32>() as u64 * 2;

    // tangent
    let attribute_3 = vertex_descriptor
        .attributes()
        .object_at(Attributes_Tangent as u64)
        .unwrap();
    attribute_3.set_format(MTLVertexFormat::Float3);
    attribute_3.set_offset(offset);
    attribute_3.set_buffer_index(BufferIndexVertices as u64);

    offset += mem::size_of::<f32>() as u64 * 3;

    // bitangent
    let attribute_4 = vertex_descriptor
        .attributes()
        .object_at(Attributes_Bitangent as u64)
        .unwrap();
    attribute_4.set_format(MTLVertexFormat::Float3);
    attribute_4.set_offset(offset);
    attribute_4.set_buffer_index(BufferIndexVertices as u64);

    offset += mem::size_of::<f32>() as u64 * 3;

    let layout_0 = vertex_descriptor
        .layouts()
        .object_at(BufferIndexVertices as u64)
        .unwrap();
    layout_0.set_stride(offset);

    vertex_descriptor
}

pub struct Textures {
    // filename: String,
    pub(crate) diffuse_texture: Option<Texture>,
    pub(crate) normal_texture: Option<Texture>,
    pub(crate) metallic_roughness_texture: Option<Texture>,
    pub(crate) ambient_occlusion_texture: Option<Texture>,
    pub(crate) emissive_texture: Option<Texture>,
}

impl Textures {
    fn new(
        diffuse_texture: Option<Texture>,
        normal_texture: Option<Texture>,
        metallic_roughness_texture: Option<Texture>,
        ambient_occlusion_texture: Option<Texture>,
        emissive_texture: Option<Texture>,
    ) -> Textures {
        Textures {
            diffuse_texture,
            normal_texture,
            metallic_roughness_texture,
            ambient_occlusion_texture,
            emissive_texture,
        }
    }
}

impl Default for Textures {
    fn default() -> Self {
        Textures {
            diffuse_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
            ambient_occlusion_texture: None,
            emissive_texture: None,
        }
    }
}
