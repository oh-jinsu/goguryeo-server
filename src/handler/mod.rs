mod incoming;
mod job;
mod selector;

use std::error::Error;
use std::collections::{BinaryHeap, HashMap};

use tokio::net::{TcpStream, TcpListener};

use crate::constants::Constants;
use crate::job::{Schedule, Job};
use crate::map::{Vector3, Map};

pub struct Context {
    constants: Constants,
    schedule_queue: BinaryHeap<Schedule<Job>>,
    listener: TcpListener,
    waitings: Vec<TcpStream>,
    connections: HashMap<[u8; 16], (TcpStream, Vector3)>,
    map: Map,
}

impl Context {
    pub fn new(constants: Constants, map: Map, listener: TcpListener) -> Self {
        Context {
            constants,
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

            if let Err(e) = job::handle(&mut self, job) {
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
