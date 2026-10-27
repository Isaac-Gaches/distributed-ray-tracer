use serde::{Deserialize, Serialize};
use crate::geometry::Geometry;
use crate::ray::{HitRecord, Ray};
use crate::vec3::Vec3;

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Object{
    pub pos: Vec3,
    pub material_id: usize,
    pub geometry: Geometry,
}

impl Object{
    pub fn hit(
        &self,
        ray: &Ray,
        t_min: f32,
        closest_t: f32,
    ) -> Option<HitRecord> {
        match &self.geometry {
            Geometry::Sphere { radius } => {
                let oc = ray.origin - self.pos;

                let a = ray.dir.dot(ray.dir);
                let half_b = oc.dot(ray.dir);
                let c = oc.dot(oc) - radius * radius;

                let discriminant = half_b * half_b - a * c;

                if discriminant < 0.0 {
                    return None;
                }

                let sqrt_d = discriminant.sqrt();

                // Try the nearer intersection first.
                let mut t = (-half_b - sqrt_d) / a;

                if t < t_min || t > closest_t {
                    t = (-half_b + sqrt_d) / a;

                    if t < t_min || t > closest_t {
                        return None;
                    }
                }

                let point = ray.at(t);
                let outward_normal = (point - self.pos) / *radius;

                Some(HitRecord::new(
                    ray,
                    point,
                    outward_normal,
                    t,
                    self.material_id,
                ))
            }

            Geometry::Quad { u, v } => {
                // The quad is:
                // pos + alpha * u + beta * v
                // where alpha and beta are between 0 and 1.

                let normal = u.cross(*v);

                // Intersect the ray with the quad's plane.
                let denom = ray.dir.dot(normal);

                // Parallel (or almost parallel) to the plane.
                if denom.abs() < 1e-8 {
                    return None;
                }

                let t = (self.pos - ray.origin).dot(normal) / denom;

                if t < t_min || t > closest_t {
                    return None;
                }

                let point = ray.at(t);
                let w = point - self.pos;

                // Express w as alpha*u + beta*v.
                // Solve the 2x2 system using dot products.
                let uu = u.dot(*u);
                let uv = u.dot(*v);
                let vv = v.dot(*v);
                let wu = w.dot(*u);
                let wv = w.dot(*v);

                let det = uu * vv - uv * uv;

                if det.abs() < 1e-12 {
                    return None;
                }

                let alpha = (wu * vv - wv * uv) / det;
                let beta = (wv * uu - wu * uv) / det;

                // The plane intersection must be inside the quad.
                if !(0.0..=1.0).contains(&alpha)
                    || !(0.0..=1.0).contains(&beta)
                {
                    return None;
                }

                let outward_normal = normal.normalize();

                Some(HitRecord::new(
                    ray,
                    point,
                    outward_normal,
                    t,
                    self.material_id,
                ))
            }
        }
    }

}
