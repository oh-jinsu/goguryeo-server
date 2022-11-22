use std::error::Error;
use std::io;

use crate::{world::{World, incoming_handler}, net::packet};

///
/// Read from a connection
/// 
pub fn handle(key: (i32, i32), context: &mut World) -> Result<(), Box<dyn Error>> {
    if let Some(conn) = context.connections.get(&key) {
        let mut buf = vec![0 as u8; 2];

        if let Err(e) = conn.try_read_one(&mut buf) {
            if e.kind() != io::ErrorKind::WouldBlock {
                eprintln!("{e}");

                World::schedule_drop(&mut context.schedule_queue, key);

                return Ok(());
            }
        
            return Ok(());
        }

        let packet = match packet::Incoming::deserialize(&mut buf) {
            Ok(packet) => packet,
            Err(e) => {
                eprintln!("{e}");

                World::schedule_drop(&mut context.schedule_queue, key);

                return Ok(());
            }
        };

        if let Err(e) = incoming_handler::handle(packet, key, context) {
            eprintln!("{e}");

            World::schedule_drop(&mut context.schedule_queue, key);

            return Ok(());
        }
    }

    Ok(())
}
