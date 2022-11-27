use serde::Deserialize;

use super::object::Object;

#[derive(Deserialize)]
pub struct Tile {
    pub id: u16,
    pub rotation: u8,

    #[serde(skip_deserializing)]
    pub object: Option<Object>
}