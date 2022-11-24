use std::error::Error;

use super::{World, job::Job};

mod welcome;
mod drop;
mod read;
mod movement;

///
/// Handle a job.
/// 
pub fn handle(context: &mut World, job: Job) -> Result<(), Box<dyn Error>> {
    match job {
        Job::Welcome(stream, id) => welcome::handle(id, stream, context),
        Job::Drop(position) => drop::handle(position, context),
        Job::Read(position) => read::handle(position, context),
        Job::Move { from, tick } => movement::handle(from, tick, context),
    }
}
