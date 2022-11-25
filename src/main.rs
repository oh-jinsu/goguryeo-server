use std::{error::Error, collections::HashMap};

use mmorpg::{world::{World, Tile}, common::math::Vector3};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let mut map = HashMap::new();
    
    for x in 0..100 {
        for z in 0..100 {
            map.insert(Vector3::new(x, 0, z), Tile { object: None });
        }
    }

    let world = World::new(map, listener);

    world.run().await
}
