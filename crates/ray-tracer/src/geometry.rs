use serde::{Deserialize, Serialize};
use crate::vec3::Vec3;

#[derive(Debug,Serialize,Deserialize,Clone)]
pub enum Geometry {
    Sphere{
        radius: f32,
    },
    Quad{
        u: Vec3,
        v: Vec3,
    },
}