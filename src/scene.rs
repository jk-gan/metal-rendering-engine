use crate::{camera::Camera, node::Node};

struct Scene {
    cameras: Vec<Camera>,
    current_camera_index: usize,
    nodes: Vec<Node>,
}

impl Scene {
    fn new() -> Self {
        Scene {
            cameras: vec![Camera::default()],
            current_camera_index: 0,
            nodes: vec![],
        }
    }

    pub fn current_camera(&self) -> &Camera {
        &self.cameras[self.current_camera_index]
    }
}
