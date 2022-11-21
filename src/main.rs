use std::{error::Error, collections::HashMap};

use mmorpg::{gatekeeper::Gatekeeper, world::{World, Tile}};
use tokio::{net::TcpListener, sync::mpsc, spawn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let (tx, rx) = mpsc::channel(16);
    
    let gatekeeper = Gatekeeper::new(listener, tx);

    spawn(async move {
        if let Err(e) = gatekeeper.run().await {
            eprintln!("{e}");
        }
    });

    let mut map = HashMap::new();
    
    for x in 0..100 {
        for y in 0..100 {
            map.insert((x, y), Tile { object: None });
        }
    }

    let world = World::new(map, rx);

    world.run().await
}
