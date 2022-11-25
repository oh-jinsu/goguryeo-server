use std::error::Error;
use std::io;

use crate::{handler::{Context, incoming}, net::{packet, io::Reader}};

///
/// Read from a connection
/// 
pub fn handle(key: [u8; 16], context: &mut Context) -> Result<(), Box<dyn Error>> {
    if let Some((stream, _)) = context.connections.get(&key) {
        let mut buf = vec![0 as u8; 2];

        if let Err(e) = stream.try_read_one(&mut buf) {
            if e.kind() != io::ErrorKind::WouldBlock {
                eprintln!("{e}");

                Context::schedule_drop(&mut context.schedule_queue, key);

                return Ok(());
            }
        
            return Ok(());
        }

        let packet = match packet::Incoming::deserialize(&mut buf) {
            Ok(packet) => packet,
            Err(e) => {
                eprintln!("{e}");

                Context::schedule_drop(&mut context.schedule_queue, key);

                return Ok(());
            }
        };

        if let Err(e) = incoming::handle(packet, key, context) {
            eprintln!("{e}");

            Context::schedule_drop(&mut context.schedule_queue, key);

            return Ok(());
        }
    }

    Ok(())
}
