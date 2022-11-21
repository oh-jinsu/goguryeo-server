use std::error::Error;
use crate::{runner::World, net::packet};

///
/// Drop a connection
/// 
pub fn handle(key: (i32, i32), context: &mut World) -> Result<(), Box<dyn Error>> {
    if let Some(conn) = context.connections.remove(&key) {
        if let Some(tile) = context.map.get_mut(&key) {
            tile.object = None;
        }

        let mut outgoing = packet::Outgoing::Disconnect { id: conn.id }.serialize();

        for (key, conn) in context.connections.iter() {
            if let Err(e) = conn.try_write_one(&mut outgoing) {
                eprintln!("{e}");

                World::schedule_drop(&mut context.schedule_queue, *key);
            }
        }
    }

    Ok(())
}