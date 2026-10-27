mod scene;
mod camera;
mod tile;
mod object;
mod material;
mod geometry;
mod vec3;
mod ray;
mod message;

use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use image::{ImageBuffer, RgbImage};
use crate::ray::Ray;

pub use crate::camera::Camera;
pub use crate::geometry::Geometry;
pub use crate::material::{Material, MaterialKind};
pub use crate::object::Object;
pub use crate::scene::Scene;
pub use crate::tile::Tile;
pub use crate::vec3::Vec3;
pub use crate::message::Message;
pub use message::{receive_message,send_message};

pub fn save_png(pixels: &[u8], width: u32, height: u32, path: &str ) -> Result<(), Box<dyn std::error::Error>> {
    let img: RgbImage = ImageBuffer::from_raw(
        width,
        height,
        pixels.to_vec(),
    )
    .ok_or("Invalid pixel buffer dimensions")?;

    img.save(path)?;

    Ok(())
}

pub fn render_tile(camera: &Camera, scene: &Scene, tile: &Tile) -> Vec<u8>{
    let mut pixels = Vec::with_capacity(
        (tile.width * tile.height * 3) as usize
    );

    for y in tile.y..tile.y + tile.height {
        for x in tile.x..tile.x + tile.width {
            let ray = camera.get_ray(x, y);
            let color = ray_colour(&ray, scene,camera);

            pixels.push((color.x * 255.999) as u8);
            pixels.push((color.y * 255.999) as u8);
            pixels.push((color.z * 255.999) as u8);
        }
    }

    pixels
}

fn ray_colour(ray: &Ray, scene: &Scene,camera: &Camera) -> Vec3 {
    const RAY_EPSILON: f32 = 0.001;

    let mut final_colour = Vec3::new(0.0, 0.0, 0.0);

    for _ in 0..camera.samples {
        let mut current_ray = Ray::new(ray.origin, ray.dir);
        let mut throughput = Vec3::new(1.0, 1.0, 1.0);
        
        for _ in 0..camera.max_bounces {
            let Some(hit) = scene.hit(&current_ray, RAY_EPSILON, f32::INFINITY) else {
                final_colour += throughput.component_mul(scene.background);
                break;
            };

            let material = &scene.materials[hit.material_id];
            throughput = throughput.component_mul(material.albedo);

            let direction = match material.kind {
                MaterialKind::Metal { roughness } => {
                    let reflected = current_ray.dir.normalize().reflect(hit.normal);
                    let rough_reflection = reflected + Vec3::random_in_unit_sphere() * roughness;

                    if rough_reflection.dot(hit.normal) > 0.0 {
                        rough_reflection.normalize()
                    } else {
                        hit.normal
                    }
                }
                MaterialKind::Diffuse => {
                    let diffuse = hit.normal + Vec3::random_unit_vector();

                    if diffuse.dot(diffuse) > 1e-12 {
                        diffuse.normalize()
                    } else {
                        hit.normal
                    }
                }
                MaterialKind::Emissive =>{
                    final_colour += material.albedo;
                    break;
                }
            };
            current_ray = Ray::new(hit.point + hit.normal * RAY_EPSILON, direction );
        }
    }
    final_colour/camera.samples as f32
}

pub fn par_render_tile( camera: &Camera, scene: &Scene, tile: &Tile,chunk_size: u32) -> Vec<u8>{
    let (result_tx,result_rx) = mpsc::channel::<(u32,Vec<u8>)>();
    let next_tile = Arc::new(AtomicUsize::new(0));
    let h = tile.height / chunk_size;
    let tiles = Arc::new((0..chunk_size).map(|i| {
        (i,Tile { x: tile.x, y: tile.y + h*i, width: tile.width, height: h })
    }).collect::<Vec<(u32,Tile)>>());

    let num_workers = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1).min(chunk_size as usize);

    thread::scope(|scope| {
        for _ in 0..num_workers {
            let result_tx = result_tx.clone();
            let next_tile = Arc::clone(&next_tile);
            let tiles = Arc::clone(&tiles);

            scope.spawn(move || {
                loop{
                    match tiles.get(next_tile.fetch_add(1,Ordering::Relaxed)){
                        Some((i,tile)) => {
                            let pixels = render_tile(&camera, &scene, &tile);

                            if result_tx.send((*i,pixels)).is_err(){
                                break; // stop thread, stopped receiving
                            }
                        }
                        None => {
                            break; // no more work
                        }
                    }
                }
            });
        }

        drop(result_tx);
    });

    let mut results: Vec<_> = result_rx.into_iter().collect();
    results.sort_by_key(|(i, _)| *i);
    results.into_iter().flat_map(|(_, pixels)| pixels).collect()
}