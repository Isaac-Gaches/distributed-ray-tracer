use std::env;
use std::net::TcpStream;
use std::time::Instant;
use ray_tracer::*;

fn main()-> std::io::Result<()> {
    let address = env::args().skip(1).next().unwrap_or_else(|| "127.0.0.1:9000".to_string());
    let mut stream = TcpStream::connect(address)?;

    let mut worker_scene = None;
    let mut worker_camera = None;

    loop{
        let message = receive_message(&mut stream)?;

        match message {
            Message::Finished => break,
            Message::RenderTile {id,tile} => {
                match (&worker_scene, &worker_camera) {
                    (Some(scene), Some(camera)) => {
                        let timer = Instant::now();
                        
                        let bytes = par_render_tile(camera,scene,&tile,16);

                        println!("Tile complete in {}s",timer.elapsed().as_secs_f32());

                        send_message(&mut stream,&Message::TileResult {
                            id,
                            bytes,
                        })?;
                    }
                    _ => {}
                }
            }
            Message::Scene {camera,scene} => {
                worker_scene = Some(scene);
                worker_camera = Some(camera);
                println!("Scene loaded");
            }
            _ => break,
        }
    }

    println!("Finished");

    Ok(())
}