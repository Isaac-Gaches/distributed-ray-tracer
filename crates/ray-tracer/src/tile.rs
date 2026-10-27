use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize,Deserialize)]
pub struct Tile{
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}