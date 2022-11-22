mod incoming_handler;
mod job_handler;
mod job;

use std::error::Error;
use std::collections::{BinaryHeap, HashMap};

use futures::future::select_all;
use tokio::sync::mpsc;
use tokio::time::{self, Instant};

use crate::common::math::Vector3;
use crate::{schedule::Schedule, net::Conn};

use self::job::Job;

pub struct Tile {
    pub object: Option<Object>
}

pub enum Object {
    Human {
        id: i32,
        state: HumanState,
    },
}

impl Object {
    pub fn new_human(id: i32) -> Self {
        Object::Human { id, state: HumanState::Idle { updated_at: None } }
    }
}

pub enum HumanState {
    Idle { updated_at: Option<Instant> },
    Move { direction: u8, updated_at: Option<Instant> },
}

pub struct World {
    schedule_queue: BinaryHeap<Schedule<Job>>,
    receiver: mpsc::Receiver<Conn>,
    connections: HashMap<Vector3, Conn>,
    map: HashMap<Vector3, Tile>,
}

impl World {
    pub fn new(map: HashMap<Vector3, Tile>, receiver: mpsc::Receiver<Conn>) -> Self {
        World {
            schedule_queue: BinaryHeap::new(),
            receiver,
            connections: HashMap::new(),
            map,
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            if let Some(job) = self.select_job().await {
                if let Err(e) = job_handler::handle(&mut self, job) {
                    eprintln!("{e}");
                }
            }
        }
    }

    async fn select_job(&mut self) -> Option<Job> {
        if self.connections.is_empty() {
            self.select_from_receiver().await
        } else if self.schedule_queue.is_empty() {
            self.select_without_schedule_queue().await
        } else {
            self.select_with_all().await
        }
    }

    async fn select_from_receiver(&mut self) -> Option<Job> {
        if let Some(conn) = self.receiver.recv().await {
            Some(Job::Welcome(conn))
        } else {
            None
        }
    }

    async fn select_without_schedule_queue(&mut self) -> Option<Job> {
        Some(tokio::select! {
            Some(conn) = self.receiver.recv() => {
                Job::Welcome(conn)
            },
            (Ok(key), _, _) = select_all(self.connections.iter_mut().map(|(key, conn)| Box::pin(async {
                conn.readable().await?;

                Ok::<&Vector3, Box<dyn Error>>(key)
            }))) => {
                Job::Read(key.clone())
            },
        })
    }

    async fn select_with_all(&mut self) -> Option<Job> {
        let first_schedule = self.schedule_queue.peek().unwrap();

        if first_schedule.deadline < time::Instant::now() {
            return Some(self.schedule_queue.pop().unwrap().job);
        }

        Some(tokio::select! {
            Some(conn) = self.receiver.recv() => {
                Job::Welcome(conn)
            },
            (Ok(key), _, _) = select_all(self.connections.iter_mut().map(|(key, conn)| Box::pin(async {
                conn.readable().await?;

                Ok::<&Vector3, Box<dyn Error>>(key)
            }))) => {
                Job::Read(key.clone())
            },
            _ = time::sleep_until(first_schedule.deadline) => {
                self.schedule_queue.pop().unwrap().job
            },
        })
    }

    fn schedule_drop(schedule_queue: &mut BinaryHeap<Schedule<Job>>, key: Vector3) {
        let job = Job::Drop(key);

        let schedule = Schedule::now(job);

        schedule_queue.push(schedule);
    }
}
