use crate::node::InnerNode;
use glam::{Mat3, Mat4, Vec3};

pub trait CameraFunction {
    fn zoom(&mut self, delta: f32);
    fn rotate(&mut self, delta: (f32, f32));
}

pub struct Camera {
    fov_degrees: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
    // projection_matrix: Mat4,
    // view_matrix: Mat4,
    inner_node: InnerNode,
}

impl Default for Camera {
    fn default() -> Self {
        let fov_degrees = 70.0;
        let aspect_ratio = 1080.0 / 720.0;
        let z_near = 0.001;
        let z_far = 100.0;
        let inner_node = InnerNode::default();

        Self::new(fov_degrees, aspect_ratio, z_near, z_far, inner_node)
    }
}

impl Camera {
    pub fn new(
        fov_degrees: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32,
        inner_node: InnerNode,
    ) -> Self {
        Self {
            fov_degrees,
            aspect_ratio,
            z_near,
            z_far,
            inner_node,
        }
    }

    pub fn fov_radians(&self) -> f32 {
        self.fov_degrees.to_radians()
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_lh(
            self.fov_radians(),
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        )
    }

    pub fn view_matrix(&self) -> Mat4 {
        let translate_matrix = Mat4::from_translation(self.inner_node.position);
        let rotate_matrix = Mat4::from_rotation_x(self.inner_node.rotation.x)
            * Mat4::from_rotation_y(self.inner_node.rotation.y)
            * Mat4::from_rotation_z(self.inner_node.rotation.z);
        let scale_matrix = Mat4::from_scale(self.inner_node.scale);
        (translate_matrix * scale_matrix * rotate_matrix).inverse()
    }
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
        self.camera.inner_node.rotation = rotation;
        self.view_matrix = self.update_view_matrix();
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.camera.inner_node.position = position;
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.camera.aspect_ratio = aspect_ratio;
    }

    pub fn rotation(&self) -> &Vec3 {
        &self.camera.inner_node.rotation
    }

    pub fn position(&self) -> &Vec3 {
        &self.camera.inner_node.position
    }

    pub fn view_matrix(&self) -> &Mat4 {
        &self.view_matrix
    }

    pub fn projection_matrix(&self) -> Mat4 {
        self.camera.projection_matrix()
    }

    fn update_view_matrix(&mut self) -> Mat4 {
        let translate_matrix = Mat4::from_translation(Vec3::new(
            self.target.x,
            self.target.y,
            self.target.z - self.distance,
        ));
        let rotate_matrix = Mat4::from_rotation_y(self.rotation().y)
            * Mat4::from_rotation_x(-self.rotation().x)
            * Mat4::from_rotation_z(0.0);
        let matrix = (rotate_matrix * translate_matrix).inverse();
        self.set_position(Mat3::from_mat4(rotate_matrix) * -matrix.col(3).truncate());

        matrix
    }
}

impl CameraFunction for ArcballCamera {
    fn zoom(&mut self, delta: f32) {
        let sensitivity = 0.05;
        self.set_distance(self.distance - (delta * sensitivity));
    }

    fn rotate(&mut self, delta: (f32, f32)) {
        let sensitivity = 0.18;
        let mut rotation = Vec3::new(self.rotation().x, self.rotation().y, self.rotation().z);

        rotation.y += delta.0.to_radians() * sensitivity;
        rotation.x += -delta.1.to_radians() * sensitivity;
        rotation.x = f32::max(
            -std::f32::consts::PI / 2.0,
            f32::min(rotation.x, std::f32::consts::PI / 2.0),
        );
        self.set_rotation(rotation);
    }
}
