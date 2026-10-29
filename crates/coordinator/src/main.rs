mod client;

use std::{env, io};
use std::net::{TcpListener};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use ray_tracer::{save_png, Camera, Geometry, Material, MaterialKind, Message, Object, Scene, Tile, Vec3};
use crate::client::handle_worker;

const CHUNKS: u32 = 32;

fn main() -> io::Result<()> {
    let workers = env::args().nth(1).unwrap_or_else(|| "1".to_string());
    let workers = workers.parse().expect("invalid number of workers");

    let listener = TcpListener::bind("0.0.0.0:9000")?;

    let (scene,camera) = init_scene();

    let height = camera.height/CHUNKS;
    let jobs = Arc::new(Mutex::new((0..CHUNKS).map(|id|{
        Message::RenderTile {
            id,
            tile: Tile {
                x: 0,
                y: height*id,
                width: camera.width,
                height,
            },
        }
    }).collect::<Vec<_>>()));

    let result = Arc::new(Mutex::new(vec![0;(camera.width*camera.height*3) as usize]));

    let mut handles = Vec::new();
    let ready = Arc::new((Mutex::new(false), Condvar::new()));

    println!("Waiting for workers...");

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                let peer = stream.peer_addr()?;
                println!("Worker connected: {peer}");

                let scene = scene.clone();
                let camera = camera.clone();
                let jobs = jobs.clone();
                let result = result.clone();
                let ready = ready.clone();

                handles.push(thread::spawn(move || {
                    if let Err(error) = handle_worker(stream,camera,scene,jobs,result,ready) {
                        eprintln!("Worker {peer} error: {error}");
                    }
                    println!("Worker disconnected: {peer}");
                }));

                if handles.len() >= workers {
                    break;
                }
            }
            Err(error) => eprintln!("Failed to accept connection: {error}"),
        }
    }

    println!("All workers connected, enter 'y' to run: ");

    let mut input = String::new();
    loop {
        input.clear();
        io::stdin().read_line(&mut input)?;

        if input.trim() == "y" {
            let (lock, condvar) = &*ready;
            *lock.lock().unwrap() = true;
            condvar.notify_all();
            break;
        }
    }

    println!("Running workers...");
    let timer = Instant::now();

    for handle in handles {
        handle.join().expect("worker thread panicked");
    }

    let image = result.lock().expect("image mutex poisoned").clone();
    save_png(&image, camera.width, camera.height, "output.png").unwrap();

    println!("Finished after: {}s", timer.elapsed().as_secs_f32());
    Ok(())
}

fn init_scene() -> (Scene,Camera) {
    let width = 8192;
    let height = 4096;
    let camera = Camera::new(
        Vec3::new(-0.5, 1.5, 5.0),
        Vec3::new(0.0, 1.0, 1.0),
        Vec3::new(0.0, 1.0, 0.0),
        50.0,
        width,
        height,
        12,
        128,
    );
    let scene = Scene {
        objects: vec![
            Object { pos: Vec3::new(0.0, 1.0, 0.0), geometry: Geometry::Sphere { radius: 1.0 }, material_id: 3 },
            Object { pos: Vec3::new(0.0, 0.5, 2.0), geometry: Geometry::Sphere { radius: 0.5 }, material_id: 1 },
            Object { pos: Vec3::new(1.5, 0.6, 1.0), geometry: Geometry::Sphere { radius: 0.6 }, material_id: 2 },
            Object { pos: Vec3::new(-1.5, 0.5, 1.0), geometry: Geometry::Sphere { radius: 0.5 }, material_id: 4 },
            Object { pos: Vec3::new(-5.0, 0.0, 5.0), geometry: Geometry::Quad { u: Vec3::new(10.0, 0.0, 0.0), v: Vec3::new(0.0, 0.0, -10.0) }, material_id: 0 },
        ],
        materials: vec![
            Material { albedo: Vec3::new(0.8, 0.3, 0.2), kind: MaterialKind::Diffuse },
            Material { albedo: Vec3::new(0.7, 0.75, 0.8), kind: MaterialKind::Metal { roughness: 0.1 } },
            Material { albedo: Vec3::new(0.8, 0.4, 0.8), kind: MaterialKind::Metal { roughness: 0.05 } },
            Material { albedo: Vec3::new(0.9, 0.9, 0.9), kind: MaterialKind::Diffuse },
            Material { albedo: Vec3::new(0.5, 0.8, 0.4), kind: MaterialKind::Emissive },
        ],
        background: Vec3::new(0.5, 0.5, 0.8),
    };

    (scene, camera)
}
