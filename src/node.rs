use crate::{camera::Camera, model::Model};
use glam::{Mat4, Vec3};
use std::rc::Rc;

// pub enum NodeType {
//     Camera(Camera),
//     Model,
// }

pub trait SceneNode {
    fn get_postion(&self) -> &Vec3;
    fn set_postion(&self);
    fn get_rotation(&self) -> &Vec3;
    fn set_rotation(&self);
    fn get_scale(&self) -> &Vec3;
    fn set_scale(&self);
}

struct TestNode {
    inner_node: InnerNode,
}

pub struct Node {
    parent: Rc<Node>,
    children: Vec<Node>,
}

pub struct InnerNode {
    pub(crate) name: String,
    pub(crate) position: Vec3,
    pub(crate) rotation: Vec3,
    pub(crate) scale: Vec3,
}

impl Default for InnerNode {
    fn default() -> Self {
        Self::new(
            "untitled".to_string(),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        )
    }
}

impl InnerNode {
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
        let rotate_matrix = Mat4::from_rotation_x(self.rotation.x)
            * Mat4::from_rotation_y(self.rotation.y)
            * Mat4::from_rotation_z(self.rotation.z);
        let scale_matrix = Mat4::from_scale(self.scale);

        translate_matrix * rotate_matrix * scale_matrix
    }
}
