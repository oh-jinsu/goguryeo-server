use std::error::Error;
use crate::job::Job;

use super::World;

mod welcome;
mod drop;
mod read;
mod movement;

///
/// Handle a job.
/// 
pub fn handle(context: &mut World, job: Job) -> Result<(), Box<dyn Error>> {
    match job {
        Job::Welcome(conn) => welcome::handle(conn, context),
        Job::Drop(key) => drop::handle(key, context),
        Job::Read(key) => read::handle(key, context),
        Job::Move { from, tick } => movement::handle(from, tick, context),
        _ => Ok(())
    }
}