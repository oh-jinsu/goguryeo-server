use std::error::Error;

use mmorpg::runner;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let world = runner::World::new(listener);

    world.run().await
}