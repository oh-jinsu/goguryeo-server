use std::error::Error;
use std::collections::{BinaryHeap, HashMap};
use std::io;

use futures::future::select_all;
use tokio::sync::mpsc;
use tokio::time::{self, Instant};

use crate::job::Job;
use crate::net::packet;
use crate::{job::Schedule, net::Conn};

pub struct Tile {
    object: Option<Object>
}

pub enum Object {
    Human {
        id: i32,
        state: HumanState,
    },
}

impl Object {
    pub fn new_human(id: i32) -> Self {
        Object::Human { id, state: HumanState::Idle }
    }
}

pub enum HumanState {
    Idle,
    Move { direction: u8 },
}

pub struct World {
    schedule_queue: BinaryHeap<Schedule>,
    receiver: mpsc::Receiver<Conn>,
    connections: HashMap<(i32, i32), Conn>,
    map: HashMap<(i32, i32), Tile>
}

impl World {
    pub fn new(receiver: mpsc::Receiver<Conn>) -> Self {
        let mut map = HashMap::new();
    
        for x in 0..100 {
            for y in 0..100 {
                map.insert((x, y), Tile { object: None });
            }
        }

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
                if let Err(e) = self.handle_job(job).await {
                    eprintln!("{e}");
                }
            }
        }
    }

    async fn select_job(&mut self) -> Option<Job> {
        if self.connections.is_empty() {
            if let Some(conn) = self.receiver.recv().await {
                Some(Job::Arrived(conn))
            } else {
                None
            }
        } else if self.schedule_queue.is_empty() {
            Some(tokio::select! {
                Some(conn) = self.receiver.recv() => {
                    Job::Arrived(conn)
                },
                (Ok(key), _, _) = select_all(self.connections.iter_mut().map(|(key, conn)| Box::pin(async {
                    conn.readable().await?;

                    Ok::<&(i32, i32), Box<dyn Error>>(key)
                }))) => {
                    Job::Read(key.clone())
                },
            })
        } else {
            Some(tokio::select! {
                Some(conn) = self.receiver.recv() => {
                    Job::Arrived(conn)
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
    }

    async fn handle_job(&mut self, job: Job) -> Result<(), Box<dyn Error>> {
        match job {
            Job::Arrived(conn) => for (key, tile) in self.map.iter_mut() {
                if let None = tile.object {
                    let id = conn.id;

                    tile.object = Some(Object::new_human(id));

                    self.connections.insert(key.clone(), conn);
                    
                    let mut outgoing = packet::Outgoing::Connect { id, x: key.0, y: key.1 }.serialize();

                    for conn in self.connections.values() {
                        conn.try_write_one(&mut outgoing)?;
                    }

                    return Ok(());
                }
            },
            Job::Drop(key) =>  if let Some(conn) = self.connections.remove(&key) {
                let id = conn.id;

                let mut outgoing = packet::Outgoing::Disconnect { id }.serialize();

                for (key, conn) in self.connections.iter() {
                    if let Err(e) = conn.try_write_one(&mut outgoing) {
                        eprintln!("{e}");

                        let job = Job::Drop(*key);

                        let schedule = Schedule::now(job);
    
                        self.schedule_queue.push(schedule);
                    }
                }
            },
            Job::Read(key) => if let Some(conn) = self.connections.get(&key) {
                let mut buf = vec![0 as u8; 2];

                if let Err(e) = conn.try_read_one(&mut buf) {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        eprintln!("{e}");

                        return self.schedule_drop(key);
                    }
                
                    return Ok(());
                }

                let packet = match packet::Incoming::deserialize(&mut buf) {
                    Ok(packet) => packet,
                    Err(e) => {
                        eprintln!("{e}");

                        return self.schedule_drop(key);
                    }
                };

                if let Err(e) = self.handle_packet(packet, key) {
                    eprintln!("{e}");

                    return self.schedule_drop(key);
                }
            },
            Job::Move { current, tick } => if let Some(tile) = self.map.get(&current) {
                if let Some(Object::Human { id, state }) = &tile.object {
                    let id = id.clone();

                    let next = match state {
                        HumanState::Move { direction } => {
                            let (x, y) = current;
            
                            match direction {
                                1 => (x, y + 1),
                                2 => (x, y - 1),
                                3 => (x - 1, y),
                                4 => (x + 1, y),
                                _ => return Ok(())
                            }
                        },
                        _ => {
                            return Ok(());
                        },
                    };

                    let is_unmovable = if let Some(tile) = self.map.get(&next) {
                        tile.object.is_some()
                    } else {
                        true
                    };

                    if is_unmovable {
                        return Ok(())
                    }

                    self.map.get_mut(&next).unwrap().object = if let Some(tile) = self.map.get_mut(&current) {
                        tile.object.take()
                    } else {
                        return Ok(());
                    };
    
                    if let Some(conn) = self.connections.remove(&current) {
                        self.connections.insert(next, conn);
                    }
    
                    let mut outgoing = packet::Outgoing::Move { id, x: next.0, y: next.1 }.serialize();
    
                    for conn in self.connections.values() {
                        conn.try_write_one(&mut outgoing)?;
                    }

                    let job = Job::Move { current: next, tick: time::Duration::from_millis(1000) };

                    let schedule = Schedule::new(job, Instant::now() + tick);

                    self.schedule_queue.push(schedule);
                }
            },
            _ => {}
        }

        Ok(())
    }

    fn handle_packet(&mut self, packet: packet::Incoming, current: (i32, i32)) -> Result<(), Box<dyn Error>> {
        match packet {
            packet::Incoming::Ping { timestamp } => if let Some(conn) = self.connections.get(&current) {
                let outgoing = packet::Outgoing::Pong { timestamp };

                conn.try_write_one(&mut outgoing.serialize())?;
            },
            packet::Incoming::Move { direction } => if let Some(tile) = self.map.get_mut(&current) {
                if let Some(Object::Human { state, .. }) = &mut tile.object {
                    if direction == 0 {
                        *state = HumanState::Idle;
                    } else {
                        *state = HumanState::Move { direction };
                    }

                    let job = Job::Move { current, tick: time::Duration::from_millis(1000) };

                    let schedule = Schedule::now(job);
    
                    self.schedule_queue.push(schedule);
                }
            },
            _ => {}
        };

        Ok(())
    }

    fn schedule_drop(&mut self, key: (i32, i32)) -> Result<(), Box<dyn Error>> {
        let job = Job::Drop(key);

        let schedule = Schedule::now(job);

        self.schedule_queue.push(schedule);

        Ok(())
    }
}
