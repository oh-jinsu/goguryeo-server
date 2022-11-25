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
        Job::Welcome(stream, key) => welcome::handle(key, stream, context),
        Job::Drop(key) => drop::handle(key, context),
        Job::Read(key) => read::handle(key, context),
        Job::Move { from, tick } => movement::handle(from, tick, context),
    }
}
