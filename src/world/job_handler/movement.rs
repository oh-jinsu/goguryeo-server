use std::error::Error;

use tokio::time;

use crate::{world::{HumanState, Object, job::Job, World}, schedule::Schedule, net::packet};

///
/// Switch the position of an object.
/// 
pub fn handle(from: (i32, i32), tick: time::Duration, context: &mut World) -> Result<(), Box<dyn Error>>  {
    let next = if let Some(Some(Object::Human { state, .. })) = context.map.get(&from).map(|tile| &tile.object) {
        match state {
            HumanState::Idle { .. } => from,
            HumanState::Move { direction, .. } => match direction {
                1 => (from.0, from.1 + 1),
                2 => (from.0, from.1 - 1),
                3 => (from.0 - 1, from.1),
                4 => (from.0 + 1, from.1),
                _ => return Ok(())
            }
        }
    } else {
        return Ok(());
    };

    let is_unmovable = if let Some(tile) = context.map.get(&next) {
        tile.object.is_some()
    } else {
        true
    };

    if is_unmovable {
        if let Some(tile) = context.map.get_mut(&from) {
            if let Some(Object::Human { id, state}) = &mut tile.object {
                let id = id.clone();
                
                *state = HumanState::Idle { updated_at: *match state {
                    HumanState::Idle { updated_at } => updated_at,
                    HumanState::Move { updated_at, .. } => updated_at,
                }};

                let mut outgoing = packet::Outgoing::Arrive { id, x: from.0, z: from.1 }.serialize();

                for (key, conn) in context.connections.iter() {
                    if let Err(e) = conn.try_write_one(&mut outgoing) {
                        eprintln!("{e}");

                        World::schedule_drop(&mut context.schedule_queue, *key);
                    }
                }
            }
        }

        return Ok(());
    }

    if let Some(tile) = context.map.get_mut(&from) {
        if let Some(Object::Human { state, id }) = &mut tile.object {
            let id = id.clone();

            if let HumanState::Move { updated_at, .. } = state {
                updated_at.replace(time::Instant::now());

                context.map.get_mut(&next).unwrap().object = tile.object.take();

                if let Some(conn) = context.connections.remove(&from) {
                    context.connections.insert(next, conn);
                }
            
                let mut outgoing = packet::Outgoing::Move { id, x: next.0, z: next.1, tick: i64::try_from(tick.as_millis()).unwrap() }.serialize();
            
                for (key, conn) in context.connections.iter() {
                    if let Err(e) = conn.try_write_one(&mut outgoing) {
                        eprintln!("{e}");
            
                        World::schedule_drop(&mut context.schedule_queue, *key);
            
                        continue;
                    }
                }
            
                let job = Job::Move { from: next, tick };
            
                context.schedule_queue.push(Schedule::new(job, time::Instant::now() + tick));
            } 
        }
    }

    Ok(())
}