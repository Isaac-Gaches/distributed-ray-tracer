use serde::{Deserialize, Serialize};
use crate::vec3::Vec3;

#[derive(Clone, Copy,Debug,Serialize,Deserialize)]
pub enum MaterialKind {
    Diffuse,
    Metal { roughness: f32 },
    Emissive,
}
#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Material {
    pub albedo: Vec3,
    pub kind: MaterialKind,
}
