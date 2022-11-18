use std::error::Error;

use mmorpg::runner;
use tokio::{net::TcpListener, sync::mpsc, spawn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let (tx, rx) = mpsc::channel(16);
    
    let gatekeeper = runner::Gatekeeper::new(listener, tx);

    spawn(async move {
        let _ = gatekeeper.run().await;
    });

    let world = runner::World::new(rx);

    world.run().await
}
