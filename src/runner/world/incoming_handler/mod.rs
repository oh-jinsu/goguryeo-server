use std::error::Error;

use crate::net::packet;

use super::World;

mod ping;
mod movement;

pub fn handle(packet: packet::Incoming, current: (i32, i32), context: &mut World) -> Result<(), Box<dyn Error>> {
    match packet {
        packet::Incoming::Ping { timestamp } => ping::handle(timestamp, current, context),
        packet::Incoming::Move { direction } => movement::handle(direction, current, context),
        _ => Ok(())
    }
}
