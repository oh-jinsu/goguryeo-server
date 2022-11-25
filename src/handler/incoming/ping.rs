use std::error::Error;

use crate::{handler::Context, net::{packet, io::Writer}};

///
/// Handle the request for ping.
/// 
/// Just return the passed timestamp to the connection.
/// 
pub fn handle(timestamp: i64, key: [u8; 16], context: &mut Context) -> Result<(), Box<dyn Error>> {
    let stream = match context.connections.get(&key) {
        Some((stream, _)) => stream,
        None => return Ok(())
    };
    
    let outgoing = packet::Outgoing::Pong { timestamp };

    stream.try_write_one(&mut outgoing.serialize())?;

    Ok(())
}
