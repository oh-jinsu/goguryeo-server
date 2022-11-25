mod incoming_handler;
mod job_handler;
mod selector;

use std::error::Error;
use std::collections::{BinaryHeap, HashMap};

use tokio::net::{TcpStream, TcpListener};
use tokio::time;

use crate::common::math::Vector3;
use crate::job::{Schedule, Job};

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
    Idle { updated_at: Option<time::Instant> },
    Move { direction: u8, updated_at: Option<time::Instant> },
}

pub struct World {
    schedule_queue: BinaryHeap<Schedule<Job>>,
    listener: TcpListener,
    waitings: Vec<TcpStream>,
    connections: HashMap<[u8; 16], (TcpStream, Vector3)>,
    map: HashMap<Vector3, Tile>,
}

impl World {
    pub fn new(map: HashMap<Vector3, Tile>, listener: TcpListener) -> Self {
        World {
            schedule_queue: BinaryHeap::new(),
            listener,
            waitings: Vec::new(),
            connections: HashMap::new(),
            map,
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let job = selector::select_job(&mut self).await;

            if let Err(e) = job_handler::handle(&mut self, job) {
                eprintln!("{e}");
            }
        }
    }

    fn schedule_drop(schedule_queue: &mut BinaryHeap<Schedule<Job>>, id: [u8; 16]) {
        let job = Job::Drop(id);

        let schedule = Schedule::now(job);

        schedule_queue.push(schedule);
    }
}
