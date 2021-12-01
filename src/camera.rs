use glam::{Mat3, Mat4, Quat, Vec3, Vec4};

struct Node {
    name: String,
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
}

impl Default for Node {
    fn default() -> Self {
        Self::new(
            "untitled".to_string(),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        )
    }
}

impl Node {
    pub fn new(name: String, position: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        Self {
            name,
            position,
            rotation,
            scale,
        }
    }

    pub fn model_matrix(&self) -> Mat4 {
        let translate_matrix = Mat4::from_translation(self.position);
        let rotate_matrix = Mat4::from_translation(self.rotation);
        let scale_matrix = Mat4::from_translation(self.scale);

        translate_matrix * rotate_matrix * scale_matrix
    }
}

pub trait CameraFunction {
    fn zoom(&mut self, delta: f32);
    fn rotate(&mut self, delta: (f32, f32));
}

struct Camera {
    fov_degrees: f32,
    fov_radians: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
    projection_matrix: Mat4,
    view_matrix: Mat4,
    node: Node,
}

impl Default for Camera {
    fn default() -> Self {
        let fov_degrees = 70.0;
        let aspect_ratio = 1.0;
        let z_near = 0.001;
        let z_far = 100.0;
        let node = Node::default();
        let projection_matrix = Mat4::perspective_lh(fov_degrees, aspect_ratio, z_near, z_far);
        let view_matrix = node.model_matrix().inverse();

        Self::new(
            fov_degrees,
            aspect_ratio,
            z_near,
            z_far,
            projection_matrix,
            view_matrix,
            node,
        )
    }
}

impl Camera {
    pub fn new(
        fov_degrees: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32,
        projection_matrix: Mat4,
        view_matrix: Mat4,
        node: Node,
    ) -> Self {
        Self {
            fov_degrees,
            fov_radians: fov_degrees.to_radians(),
            aspect_ratio,
            z_near,
            z_far,
            projection_matrix,
            view_matrix,
            node,
        }
    }

    // pub fn zoom(&self, delta: f32) {}
    // pub fn rotate(&self, delta: (f32, f32)) {}
}

pub struct ArcballCamera {
    min_distance: f32,
    max_distance: f32,
    target: Vec3,
    distance: f32,
    camera: Camera,
    view_matrix: Mat4,
}

impl ArcballCamera {
    pub fn new(min_distance: f32, max_distance: f32, target: Vec3, distance: f32) -> Self {
        let mut camera = Self {
            min_distance,
            max_distance,
            target,
            distance,
            camera: Camera::default(),
            view_matrix: Mat4::IDENTITY,
        };

        camera.view_matrix = camera.update_view_matrix();
        camera
    }

    pub fn set_target(&mut self, target: Vec3) {
        self.target = target;
        self.view_matrix = self.update_view_matrix();
    }

    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance;
        self.view_matrix = self.update_view_matrix();
    }

    pub fn set_rotation(&mut self, rotation: Vec3) {
        self.camera.node.rotation = rotation;
        self.view_matrix = self.update_view_matrix();
    }

    fn set_position(&mut self, position: Vec3) {
        self.camera.node.position = position;
    }

    pub fn rotation(&self) -> &Vec3 {
        &self.camera.node.rotation
    }

    pub fn view_matrix(&self) -> &Mat4 {
        &self.view_matrix
    }

    pub fn projection_matrix(&self) -> &Mat4 {
        &self.camera.projection_matrix
    }

    fn update_view_matrix(&mut self) -> Mat4 {
        let translate_matrix = Mat4::from_translation(Vec3::new(
            self.target.x,
            self.target.y,
            self.target.z - self.distance,
        ));
        let rotate_matrix = Mat4::from_quat(Quat::from_vec4(Vec4::new(
            -self.rotation().x,
            self.rotation().y,
            0.0,
            0.0,
        )));
        let matrix = (rotate_matrix * translate_matrix).inverse();
        self.set_position(Mat3::from_mat4(rotate_matrix) * matrix.col(2).truncate());

        matrix
    }
}

impl CameraFunction for ArcballCamera {
    fn zoom(&mut self, delta: f32) {
        let sensitivity = 0.05;
        self.set_distance(self.distance - (delta * sensitivity))
    }

    fn rotate(&mut self, delta: (f32, f32)) {
        let sensitivity = 0.005;

        let mut rotation = Vec3::new(self.rotation().x, self.rotation().y, self.rotation().z);
        rotation.y += delta.0 * sensitivity;
        rotation.x += delta.1 * sensitivity;
        rotation.x = f32::max(
            -std::f32::consts::PI / 2.0,
            f32::min(rotation.x, std::f32::consts::PI / 2.0),
        );
        self.set_rotation(rotation);
    }
}
