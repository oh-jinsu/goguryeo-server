use std::error::Error;

use crate::net::packet;

use super::Context;

mod ping;
mod movement;

pub fn handle(packet: packet::Incoming, key: [u8; 16], context: &mut Context) -> Result<(), Box<dyn Error>> {
    match packet {
        packet::Incoming::Ping { timestamp } => ping::handle(timestamp, key, context),
        packet::Incoming::Move { direction } => movement::handle(direction, key, context),
        _ => Ok(())
    }
}
