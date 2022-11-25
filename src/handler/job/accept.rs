use std::error::Error;
use tokio::net::TcpStream;

use crate::handler::Context;

pub fn handle(stream: TcpStream, context: &mut Context) -> Result<(), Box<dyn Error>> {
    context.waitings.push(stream);

    Ok(())
}