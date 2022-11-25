use std::error::Error;
use crate::{handler::Context, net::{packet, io::Writer}, map::object::Object};

///
/// Drop a connection
/// 
pub fn handle(id: [u8; 16], context: &mut Context) -> Result<(), Box<dyn Error>> {
    if let Some((_, position)) = context.connections.remove(&id) {
        if let Some(tile) = context.map.get_mut(&position) {
            if let Some(Object::Human { id: object_id, .. }) = &tile.object {
                if id == *object_id {
                    tile.object = None;
                }
            }
        }

        let mut outgoing = packet::Outgoing::Disconnect { id }.serialize();

        for (key, (stream, _)) in context.connections.iter() {
            if let Err(e) = stream.try_write_one(&mut outgoing) {
                eprintln!("{e}");

                Context::schedule_drop(&mut context.schedule_queue, *key);
            }
        }
    }

    Ok(())
}
