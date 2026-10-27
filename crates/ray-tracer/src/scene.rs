use serde::{Deserialize, Serialize};
use crate::material::Material;
use crate::object::Object;
use crate::ray::{HitRecord, Ray};
use crate::vec3::Vec3;

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Scene{
    pub objects: Vec<Object>,
    pub materials: Vec<Material>,
    pub background: Vec3,
}

impl Scene {
    pub fn hit(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<HitRecord> {
        let mut closest_t = t_max;
        let mut closest_hit = None;

        for object in &self.objects {
            if let Some(hit) = object.hit(ray, t_min, closest_t) {
                closest_t = hit.t;
                closest_hit = Some(hit);
            }
        }

        closest_hit
    }
}

