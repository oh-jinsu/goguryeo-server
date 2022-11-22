use std::error::Error;

use crate::{net::packet, common::math::Vector3};

use super::World;

mod ping;
mod movement;

pub fn handle(packet: packet::Incoming, position: Vector3, context: &mut World) -> Result<(), Box<dyn Error>> {
    match packet {
        packet::Incoming::Ping { timestamp } => ping::handle(timestamp, position, context),
        packet::Incoming::Move { direction } => movement::handle(direction, position, context),
        _ => Ok(())
    }
}
