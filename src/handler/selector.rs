use std::{error::Error, collections::HashMap};
use std::collections::BinaryHeap;

use futures::future::select_all;
use tokio::{time, net::TcpStream};

use crate::job::{Job, Schedule};
use crate::map::Vector3;

use super::Context;

pub async fn select_job(context: &mut Context) -> Job {
    if let Some(job) = get_late_schedule(&mut context.schedule_queue) {
        return job;
    }
    
    tokio::select! {
        Ok((stream, _)) = context.listener.accept() => {
            Job::Accept(stream)
        },
        Ok(_) = wait_first_schedule(&context.schedule_queue) => {
            context.schedule_queue.pop().unwrap().job
        },
        Ok(index) = select_from_waitings(&mut context.waitings) => {
            Job::Auth(index)
        }
        Ok(id) = select_from_connections(&mut context.connections) => {
            Job::Read(id.clone())
        },
    }
}

fn get_late_schedule(schedule_queue: &mut BinaryHeap<Schedule<Job>>) -> Option<Job> {
    if schedule_queue.is_empty() {
        return None
    }

    let first_schedule = schedule_queue.peek().unwrap();

    if first_schedule.deadline > time::Instant::now() {
        return None;
    }

    Some(schedule_queue.pop().unwrap().job)
} 

async fn wait_first_schedule(schedule_queue: &BinaryHeap<Schedule<Job>>) -> Result<(), Box<dyn Error>> {
    if schedule_queue.is_empty() {
        return Err("no schedule".into());
    }
    
    let first_schedule = schedule_queue.peek().unwrap();

    time::sleep_until(first_schedule.deadline).await;

    Ok(())
}

async fn select_from_waitings(waitings: &mut Vec<TcpStream>) -> Result<usize, Box<dyn Error>> {
    if waitings.is_empty() {
        return Err("no waiting".into());
    }

    match select_all(waitings.iter_mut().enumerate().map(|(index, stream)| Box::pin(async move {
        stream.readable().await?;

        Ok::<usize, Box<dyn Error>>(index)
    }))).await {
        (Ok(index), _, _) => Ok(index),
        (Err(e), _, _) => Err(e),
    }
}

async fn select_from_connections(connections: &mut HashMap<[u8; 16], (TcpStream, Vector3)>) -> Result<&[u8; 16], Box<dyn Error>> {
    if connections.is_empty() {
        return Err("no connections".into())
    }

    match select_all(connections.iter_mut().map(|(id, (stream, _))| Box::pin(async {
        stream.readable().await?;

        Ok::<&[u8; 16], Box<dyn Error>>(id)
    }))).await {
        (Ok(id), _, _) => Ok(id),
        (Err(e), _, _) => Err(e),
    }
}
