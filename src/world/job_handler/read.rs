use std::error::Error;
use std::io;

use crate::{world::{World, incoming_handler}, net::packet, common::math::Vector3};

///
/// Read from a connection
/// 
pub fn handle(position: Vector3, context: &mut World) -> Result<(), Box<dyn Error>> {
    if let Some(conn) = context.connections.get(&position) {
        let mut buf = vec![0 as u8; 2];

        if let Err(e) = conn.try_read_one(&mut buf) {
            if e.kind() != io::ErrorKind::WouldBlock {
                eprintln!("{e}");

                World::schedule_drop(&mut context.schedule_queue, position);

                return Ok(());
            }
        
            return Ok(());
        }

        let packet = match packet::Incoming::deserialize(&mut buf) {
            Ok(packet) => packet,
            Err(e) => {
                eprintln!("{e}");

                World::schedule_drop(&mut context.schedule_queue, position);

                return Ok(());
            }
        };

        if let Err(e) = incoming_handler::handle(packet, position, context) {
            eprintln!("{e}");

            World::schedule_drop(&mut context.schedule_queue, position);

            return Ok(());
        }
    }

    Ok(())
}
