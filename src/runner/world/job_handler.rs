use std::{error::Error, io};
use tokio::time;

use crate::{job::{Job, Schedule}, net::{packet, Conn}};

use super::{World, Object, HumanState, incoming_handler};

///
/// Handle a job.
/// 
pub fn handle(context: &mut World, job: Job) -> Result<(), Box<dyn Error>> {
    match job {
        Job::Welcome(conn) => handle_welcome(conn, context),
        Job::Drop(key) => handle_drop(key, context),
        Job::Read(key) => handle_read(key, context),
        Job::Move { from, tick } => handle_move(from, tick, context),
        _ => Ok(())
    }
}

/// 
/// Welcome a conection.
/// 
pub fn handle_welcome(conn: Conn, context: &mut World) -> Result<(), Box<dyn Error>> {
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

///
/// Drop a connection
/// 
pub fn handle_drop(key: (i32, i32), context: &mut World) -> Result<(), Box<dyn Error>> {
    if let Some(conn) = context.connections.remove(&key) {
        if let Some(tile) = context.map.get_mut(&key) {
            tile.object = None;
        }

        let id = conn.id;

        let mut outgoing = packet::Outgoing::Disconnect { id }.serialize();

        for (key, conn) in context.connections.iter() {
            if let Err(e) = conn.try_write_one(&mut outgoing) {
                eprintln!("{e}");

                World::schedule_drop(&mut context.schedule_queue, *key);
            }
        }
    }

    Ok(())
}

///
/// Read from a connection
/// 
pub fn handle_read(key: (i32, i32), context: &mut World) -> Result<(), Box<dyn Error>> {
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

///
/// Switch the position of an object.
/// 
pub fn handle_move(from: (i32, i32), tick: time::Duration, context: &mut World) -> Result<(), Box<dyn Error>>  {
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
            
                println!("{:?}", context.pointed_at.take().map(|at| at.elapsed()) );
            
                let job = Job::Move { from: next, tick };
            
                context.schedule_queue.push(Schedule::new(job, time::Instant::now() + tick));
            } 
        }
    }

    Ok(())
}