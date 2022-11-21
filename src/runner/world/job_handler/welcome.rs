use std::error::Error;
use crate::{net::{Conn, packet}, runner::{World, world::Object}};

/// 
/// Welcome a conection.
/// 
pub fn handle(conn: Conn, context: &mut World) -> Result<(), Box<dyn Error>> {
    for (current, tile) in context.map.iter_mut() {
        if let None = tile.object {
            let id = conn.id;

            tile.object = Some(Object::new_human(id));
            
            let mut users = vec![(id, current.0, current.1)];

            let mut connect = packet::Outgoing::Connect { id, x: current.0, z: current.1 }.serialize();

            for (other, conn) in context.connections.iter() {
                if let Err(e) = conn.try_write_one(&mut connect) {
                    eprintln!("{e}");

                    World::schedule_drop(&mut context.schedule_queue, *other);

                    continue;
                }

                users.push((conn.id, other.0, other.1));
            }

            let mut introduce = packet::Outgoing::Introduce { users }.serialize();

            if let Err(e) = conn.try_write_one(&mut introduce) {
                eprintln!("{e}");

                World::schedule_drop(&mut context.schedule_queue, *current);

                return Ok(());
            }

            context.connections.insert(current.clone(), conn);
            
            return Ok(());
        }
    }

    Ok(())
}