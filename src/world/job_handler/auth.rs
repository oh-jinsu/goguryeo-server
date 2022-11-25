use std::{error::Error, io};

use crate::{world::World, net::{io::*, packet}, auth, job::{Schedule, Job}};

pub fn handle(index: usize, context: &mut World) -> Result<(), Box<dyn Error>> {
    let stream = match context.waitings.get(index) {
        Some(stream) => stream,
        None => return Ok(())
    };

    let mut buf = vec![0 as u8; 2];

    if let Err(e) = stream.try_read_one(&mut buf) {
        if e.kind() != io::ErrorKind::WouldBlock {
            eprintln!("{e}");

            context.waitings.remove(index);

            return Ok(());
        }
    
        return Ok(());
    }

    let token = match packet::Incoming::deserialize(&mut buf) {
        Ok(packet::Incoming::Hello { token }) => token,
        Ok(_) => {
            eprintln!("auth interrupted");

            context.waitings.remove(index);

            return Ok(());
        }
        Err(e) => {
            eprintln!("{e}");

            context.waitings.remove(index);

            return Ok(());
        }
    };

    let secret = std::env::var("AUTH_SECRET").unwrap();

    let token = auth::verify(&token, &secret)?;

    let outgoing = packet::Outgoing::Hello { id: token.id };

    stream.try_write_one(&mut outgoing.serialize())?;

    let stream = context.waitings.remove(index);

    let schedule = Schedule::now(Job::Welcome(stream, token.id));

    context.schedule_queue.push(schedule);
    
    Ok(())
}