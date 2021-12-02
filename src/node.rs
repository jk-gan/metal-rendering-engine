use glam::{Mat4, Vec3};

pub struct Node {
    pub(crate) name: String,
    pub(crate) position: Vec3,
    pub(crate) rotation: Vec3,
    pub(crate) scale: Vec3,
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
