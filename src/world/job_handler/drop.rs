use std::error::Error;
use crate::{world::World, net::{packet, Writer}, common::math::Vector3};

///
/// Drop a connection
/// 
pub fn handle(position: Vector3, context: &mut World) -> Result<(), Box<dyn Error>> {
    if let Some((_, id)) = context.connections.remove(&position) {
        if let Some(tile) = context.map.get_mut(&position) {
            tile.object = None;
        }

        let mut outgoing = packet::Outgoing::Disconnect { id }.serialize();

        for (key, (stream, _)) in context.connections.iter() {
            if let Err(e) = stream.try_write_one(&mut outgoing) {
                eprintln!("{e}");

                World::schedule_drop(&mut context.schedule_queue, *key);
            }
        }
    }

    Ok(())
}
