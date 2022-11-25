use std::error::Error;
use tokio::net::TcpStream;

use crate::world::World;

pub fn handle(stream: TcpStream, context: &mut World) -> Result<(), Box<dyn Error>> {
    context.waitings.push(stream);

    Ok(())
}