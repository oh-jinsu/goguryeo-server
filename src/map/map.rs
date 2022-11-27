use std::collections::HashMap;

use serde::Deserialize;

use super::{tile::Tile, Vector3};

#[derive(Deserialize)]
pub struct Map {
    pub id: String,
    pub version: u16,
    pub name: String,
    pub tiles: HashMap<Vector3, Tile>
}