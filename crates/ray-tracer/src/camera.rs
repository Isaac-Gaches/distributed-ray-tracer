use serde::{Deserialize, Serialize};
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Camera{
    pub width: u32,
    pub height: u32,
    pub max_bounces: u32,
    pub samples: u32,
    
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    half_width: f32,
    half_height: f32,
    pos: Vec3
}

impl Camera{
    pub fn new(pos: Vec3, look_at: Vec3,up: Vec3,fov: f32, width: u32, height: u32, max_bounces: u32,samples: u32)-> Self{
        let forward = (look_at-pos).normalize();
        let right = forward.cross(up).normalize();
        let up = right.cross(forward).normalize();

        let theta = fov.to_radians();
        let half_height = (theta * 0.5).tan();
        let aspect = width as f32 / height as f32;
        let half_width = aspect * half_height;
        
        Self{
            width,
            height,
            max_bounces,
            samples,
            forward,
            right,
            up,
            half_width,
            half_height,
            pos,
        }
    }
    pub fn get_ray(&self, x: u32, y: u32) -> Ray{
        let u = (x as f32 + 0.5) / self.width as f32;
        let v = (y as f32 + 0.5) / self.height as f32;
        let sx = (2.0 * u - 1.0) * self.half_width;
        let sy = (1.0 - 2.0 * v) * self.half_height;

        let direction = (self.forward + self.right * sx + self.up * sy).normalize();

        Ray::new(self.pos, direction)
    }
}
