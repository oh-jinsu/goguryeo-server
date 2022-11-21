use std::error::Error;
use tokio::time;

use crate::{net::packet, job::{Schedule, Job}};

use super::{World, Object, HumanState};

pub fn handle(packet: packet::Incoming, current: (i32, i32), context: &mut World) -> Result<(), Box<dyn Error>> {
    match packet {
        packet::Incoming::Ping { timestamp } => handle_ping(timestamp, current, context),
        packet::Incoming::Move { direction } => handle_move(direction, current, context),
        _ => Ok(())
    }
}

///
/// Handle the request for ping.
/// 
/// Just return the passed timestamp to the connection.
/// 
pub fn handle_ping(timestamp: i64, current: (i32, i32), context: &mut World) -> Result<(), Box<dyn Error>> {
    let conn = match context.connections.get(&current) {
        Some(conn) => conn,
        None => return Ok(())
    };

    let outgoing = packet::Outgoing::Pong { timestamp };

    conn.try_write_one(&mut outgoing.serialize())?;

    Ok(())
}

///
/// Handle the request for move.
/// 
/// Change the state of the human object, and
/// let a job execute the actual position swtiching.
/// 
pub fn handle_move(direction: u8, current: (i32, i32), context: &mut World) -> Result<(), Box<dyn Error>> {
    context.pointed_at = Some(time::Instant::now());

    if let Some(tile) = context.map.get_mut(&current) {
        if let Some(Object::Human { state, .. }) = &mut tile.object {
            if direction == 0 {
                *state = HumanState::Idle { updated_at: *match state {
                    HumanState::Idle { updated_at } => updated_at,
                    HumanState::Move { updated_at, .. } => updated_at,
                }};
            } else {
                if let HumanState::Move { direction: old, updated_at } = state {
                    if *old != direction {
                        *state = HumanState::Move { direction, updated_at: *updated_at };
                    }

                    return Ok(());
                };

                let tick = time::Duration::from_millis(300);

                let now = time::Instant::now();

                if let Some(updated_at) = match state {
                    HumanState::Idle { updated_at } => updated_at,
                    HumanState::Move { updated_at, .. } => updated_at,
                } {
                    if now < updated_at.to_owned() + tick {
                        return Ok(())
                    }
                }

                *state = HumanState::Move { direction, updated_at: Some(now) };

                let job = Job::Move { from: current, tick };

                context.schedule_queue.push(Schedule::now(job));
            }
        }
    }

    Ok(())
}