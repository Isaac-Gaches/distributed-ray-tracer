use crate::vec3::Vec3;

pub struct Ray{
    pub(crate) origin: Vec3,
    pub(crate) dir: Vec3,
}
impl Ray{
    pub fn new(origin: Vec3, dir: Vec3) -> Ray{
        Ray{
            origin,
            dir,
        }
    }
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.dir * t
    }
}
pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub front_face: bool,
    pub material_id: usize,
}
impl HitRecord {
    pub fn new(
        ray: &Ray,
        point: Vec3,
        outward_normal: Vec3,
        t: f32,
        material_id: usize,
    ) -> Self {
        let front_face = ray.dir.dot(outward_normal) < 0.0;

        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        Self {
            point,
            normal,
            t,
            front_face,
            material_id,
        }
    }
}