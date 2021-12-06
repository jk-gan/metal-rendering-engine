use crate::shader_bindings::{
    Light, LightType_Ambientlight, LightType_Pointlight, LightType_Spotlight, LightType_Sunlight,
};
use glam::Vec3A;

pub struct Lighting {
    pub lights: Vec<Light>,
    pub count: u32,
}

impl Lighting {
    pub fn new() -> Lighting {
        let sunlight = {
            let mut light = Self::build_default_light();
            light.position = unsafe { std::mem::transmute(Vec3A::new(0.4, 1.5, -2.0)) };
            light
        };
        let ambient_light = {
            let mut light = Self::build_default_light();
            light.color = unsafe { std::mem::transmute(Vec3A::new(0.3, 0.3, 0.3)) };
            light.intensity = 0.2;
            light.type_ = LightType_Ambientlight;
            light
        };
        let fill_light = {
            let mut light = Self::build_default_light();
            light.position = unsafe { std::mem::transmute(Vec3A::new(0.0, -0.1, 0.4)) };
            light.specularColor = unsafe { std::mem::transmute(Vec3A::new(0.0, 0.0, 0.0)) };
            light.color = unsafe { std::mem::transmute(Vec3A::new(0.4, 0.4, 0.4)) };
            light
        };
        let lights = vec![sunlight, ambient_light, fill_light];
        let count = lights.len() as u32;

        Self { lights, count }
    }

    fn build_default_light() -> Light {
        unsafe {
            Light {
                position: std::mem::transmute(Vec3A::new(0.0, 0.0, 0.0)),
                color: std::mem::transmute(Vec3A::new(1.0, 1.0, 1.0)),
                specularColor: std::mem::transmute(Vec3A::new(1.0, 1.0, 1.0)),
                intensity: 0.6,
                attenuation: std::mem::transmute(Vec3A::new(1.0, 0.0, 0.0)),
                type_: LightType_Sunlight,
                coneAngle: 0.0,
                coneDirection: std::mem::transmute(Vec3A::new(0.0, 0.0, 0.0)),
                coneAttenuation: 0.0,
                __bindgen_padding_0: std::mem::zeroed(),
                __bindgen_padding_1: std::mem::zeroed(),
            }
        }
    }
}
