use std::io;
use std::net::TcpStream;
use std::sync::{Arc, Condvar, Mutex};
use ray_tracer::{receive_message, send_message, Camera, Message, Scene};
use crate::CHUNKS;

pub fn handle_worker(
    mut stream: TcpStream,
    camera: Camera,
    scene: Scene,
    jobs: Arc<Mutex<Vec<Message>>>,
    result: Arc<Mutex<Vec<u8>>>,
    ready: Arc<(Mutex<bool>,Condvar)>,
) -> io::Result<()> {
    let image_width = camera.width;
    let chunk_height = camera.height/CHUNKS;

    send_message(&mut stream,&Message::Scene {
        camera,
        scene,
    })?;

    let (lock, condvar) = &*ready;
    let mut started = lock.lock().unwrap();

    while !*started {
        started = condvar.wait(started).unwrap();
    }
    drop(started);

    match jobs.lock().unwrap().pop() {
        Some(job) => {
            send_message(&mut stream,&job)?;
        }
        None => {
            send_message(&mut stream,&Message::Finished)?;
            return Ok(());
        }
    }

    loop{
        let message = receive_message(&mut stream)?;

        match message {
            Message::TileResult {id,bytes} =>{
                let bytes_per_row = image_width * 3;
                let start = (id * bytes_per_row * chunk_height) as usize;
                let end = start + (chunk_height * bytes_per_row) as usize;

                result.lock().unwrap()[start..end].copy_from_slice(&bytes);

                let job = jobs.lock().unwrap().pop();
                match job {
                    Some(job) => {
                        send_message(&mut stream,&job)?;
                    }
                    None => {
                        send_message(&mut stream,&Message::Finished)?;
                        break;
                    }
                }
            }
            _=>{}
        }
    }

    Ok(())
}