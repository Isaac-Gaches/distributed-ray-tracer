use serde::{Deserialize, Serialize};
use crate::{Camera, Scene, Tile};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use serde::{de::DeserializeOwned};

const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024; // 16 MiB

#[derive(Debug, Serialize, Deserialize)]
pub enum Message{
    Scene {
        camera: Camera,
        scene: Scene,
    },
    RenderTile {
        id: u32,
        tile: Tile,
    },
    TileResult {
        id: u32,
        bytes: Vec<u8>,
    },
    Finished,
}

pub fn send_message<T: Serialize>(stream: &mut TcpStream, message: &T) -> io::Result<()> {
    let payload = bincode::serialize(message)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if payload.len() > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "message too large",
        ));
    }

    let length = u32::try_from(payload.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "message too large"))?;

    stream.write_all(&length.to_be_bytes())?;
    stream.write_all(&payload)?;
    Ok(())
}

pub fn receive_message<T: DeserializeOwned>(stream: &mut TcpStream) -> io::Result<T> {
    let mut length_bytes = [0_u8; 4];
    stream.read_exact(&mut length_bytes)?;

    let length = u32::from_be_bytes(length_bytes) as usize;
    if length > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "message too large",
        ));
    }

    let mut payload = vec![0_u8; length];
    stream.read_exact(&mut payload)?;

    bincode::deserialize(&payload)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}