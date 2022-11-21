mod incoming_handler;
mod job_handler;

use std::error::Error;
use std::collections::{BinaryHeap, HashMap};

use futures::future::select_all;
use tokio::sync::mpsc;
use tokio::time::{self, Instant};

use crate::job::Job;
use crate::{job::Schedule, net::Conn};

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
    schedule_queue: BinaryHeap<Schedule>,
    receiver: mpsc::Receiver<Conn>,
    connections: HashMap<(i32, i32), Conn>,
    map: HashMap<(i32, i32), Tile>,
    pointed_at: Option<time::Instant>,
}

impl World {
    pub fn new(map: HashMap<(i32, i32), Tile>, receiver: mpsc::Receiver<Conn>) -> Self {
        World {
            schedule_queue: BinaryHeap::new(),
            receiver,
            connections: HashMap::new(),
            map,
            pointed_at: None,
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

                Ok::<&(i32, i32), Box<dyn Error>>(key)
            }))) => {
                Job::Read(key.clone())
            },
        })
    }

    async fn select_with_all(&mut self) -> Option<Job> {
        Some(tokio::select! {
            Some(conn) = self.receiver.recv() => {
                Job::Welcome(conn)
            },
            (Ok(key), _, _) = select_all(self.connections.iter_mut().map(|(key, conn)| Box::pin(async {
                conn.readable().await?;

                Ok::<&(i32, i32), Box<dyn Error>>(key)
            }))) => {
                Job::Read(key.clone())
            },
            _ = time::sleep_until(self.schedule_queue.peek().unwrap().deadline) => {
                self.schedule_queue.pop().unwrap().job
            },
        })
    }

    fn schedule_drop(schedule_queue: &mut BinaryHeap<Schedule>, key: (i32, i32)) {
        let job = Job::Drop(key);

        let schedule = Schedule::now(job);

        schedule_queue.push(schedule);
    }
}
