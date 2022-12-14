use std::error::Error;
use tokio::time;

use crate::{handler::Context, job::{Schedule, Job}, map::object::{Object, HumanState}};

///
/// Handle the request for move.
/// 
/// Change the state of the human object, and
/// let a job execute the actual position swtiching.
/// 
pub fn handle(direction: u8, key: [u8; 16], context: &mut Context) -> Result<(), Box<dyn Error>> {
    let (_, position) = match context.connections.get(&key) {
        Some(conn) => conn,
        None => return Err("connectio not found".into())
    };
    
    if let Some(tile) = context.map.get_mut(&position) {
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

                let job = Job::Move { from: *position, tick };

                context.schedule_queue.push(Schedule::now(job));
            }
        }
    }

    Ok(())
}
