mod incoming_handler;
mod job_handler;
mod job;

use std::error::Error;
use std::collections::{BinaryHeap, HashMap};

use futures::future::select_all;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{self, Instant};

use crate::common::math::Vector3;
use crate::schedule::Schedule;

use self::job::Job;

pub struct Tile {
    pub object: Option<Object>
}

pub enum Object {
    Human {
        id: [u8; 16],
        state: HumanState,
    },
}

impl Object {
    pub fn new_human(id: [u8; 16]) -> Self {
        Object::Human { id, state: HumanState::Idle { updated_at: None } }
    }
}

pub enum HumanState {
    Idle { updated_at: Option<Instant> },
    Move { direction: u8, updated_at: Option<Instant> },
}

pub struct World {
    schedule_queue: BinaryHeap<Schedule<Job>>,
    receiver: mpsc::Receiver<(TcpStream, [u8; 16])>,
    connections: HashMap<[u8; 16], (TcpStream, Vector3)>,
    map: HashMap<Vector3, Tile>,
}

impl World {
    pub fn new(map: HashMap<Vector3, Tile>, receiver: mpsc::Receiver<(TcpStream, [u8; 16])>) -> Self {
        World {
            schedule_queue: BinaryHeap::new(),
            receiver,
            connections: HashMap::new(),
            map,
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let job = self.select_job().await;

            if let Err(e) = job_handler::handle(&mut self, job) {
                eprintln!("{e}");
            }
        }
    }

    async fn select_job(&mut self) -> Job {
        if let Some(job) = Self::get_late_schedule(&mut self.schedule_queue) {
            return job;
        }
        
        tokio::select! {
            Some((stream, id)) = self.receiver.recv() => {
                Job::Welcome(stream, id)
            },
            Ok(id) = Self::select_from_connections(&mut self.connections) => {
                Job::Read(id.clone())
            },
            Ok(_) = Self::select_from_schedule(&self.schedule_queue) => {
                self.schedule_queue.pop().unwrap().job
            }
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

    async fn select_from_schedule(schedule_queue: &BinaryHeap<Schedule<Job>>) -> Result<(), Box<dyn Error>> {
        if schedule_queue.is_empty() {
            return Err("no schedule".into());
        }
        
        let first_schedule = schedule_queue.peek().unwrap();

        time::sleep_until(first_schedule.deadline).await;
 
        Ok(())
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

    fn schedule_drop(schedule_queue: &mut BinaryHeap<Schedule<Job>>, id: [u8; 16]) {
        let job = Job::Drop(id);

        let schedule = Schedule::now(job);

        schedule_queue.push(schedule);
    }
}
