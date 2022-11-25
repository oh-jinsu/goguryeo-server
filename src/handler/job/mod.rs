use std::error::Error;

use crate::job::Job;

use super::Context;

mod accept;
mod auth;
mod welcome;
mod drop;
mod read;
mod movement;

///
/// Handle a job.
/// 
pub fn handle(context: &mut Context, job: Job) -> Result<(), Box<dyn Error>> {
    match job {
        Job::Accept(stream) => accept::handle(stream, context),
        Job::Auth(index) => auth::handle(index, context),
        Job::Welcome(stream, key) => welcome::handle(key, stream, context),
        Job::Drop(key) => drop::handle(key, context),
        Job::Read(key) => read::handle(key, context),
        Job::Move { from, tick } => movement::handle(from, tick, context),
    }
}
