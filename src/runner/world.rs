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
        Object::Human { id, state: HumanState::Idle { moved_at: None } }
    }
}

pub enum HumanState {
    Idle { moved_at: Option<Instant> },
    Move { direction: u8, moved_at: Option<Instant> },
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
            Job::Arrived(conn) => for (current, tile) in self.map.iter_mut() {
                if let None = tile.object {
                    let id = conn.id;

                    tile.object = Some(Object::new_human(id));
                   
                    let mut users = vec![(id, current.0, current.1)];

                    let mut connect = packet::Outgoing::Connect { id, x: current.0, z: current.1 }.serialize();

                    for (other, conn) in self.connections.iter() {
                        if let Err(e) = conn.try_write_one(&mut connect) {
                            eprintln!("{e}");
    
                            Self::schedule_drop(&mut self.schedule_queue, *other);

                            continue;
                        }

                        users.push((conn.id, other.0, other.1));
                    }

                    let mut introduce = packet::Outgoing::Introduce { users }.serialize();

                    if let Err(e) = conn.try_write_one(&mut introduce) {
                        eprintln!("{e}");

                        Self::schedule_drop(&mut self.schedule_queue, *current);

                        return Ok(());
                    }

                    self.connections.insert(current.clone(), conn);
                    
                    return Ok(());
                }
            },
            Job::Drop(key) => if let Some(conn) = self.connections.remove(&key) {
                if let Some(tile) = self.map.get_mut(&key) {
                    tile.object = None;
                }

                let id = conn.id;

                let mut outgoing = packet::Outgoing::Disconnect { id }.serialize();

                for (key, conn) in self.connections.iter() {
                    if let Err(e) = conn.try_write_one(&mut outgoing) {
                        eprintln!("{e}");

                        Self::schedule_drop(&mut self.schedule_queue, *key);
                    }
                }
            },
            Job::Read(key) => if let Some(conn) = self.connections.get(&key) {
                let mut buf = vec![0 as u8; 2];

                if let Err(e) = conn.try_read_one(&mut buf) {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        eprintln!("{e}");

                        Self::schedule_drop(&mut self.schedule_queue, key);

                        return Ok(());
                    }
                
                    return Ok(());
                }

                let packet = match packet::Incoming::deserialize(&mut buf) {
                    Ok(packet) => packet,
                    Err(e) => {
                        eprintln!("{e}");

                        Self::schedule_drop(&mut self.schedule_queue, key);

                        return Ok(());
                    }
                };

                if let Err(e) = self.handle_packet(packet, key) {
                    eprintln!("{e}");

                    Self::schedule_drop(&mut self.schedule_queue, key);

                    return Ok(());
                }
            },
            Job::KeepMove { from, tick } => if let Some(tile) = self.map.get_mut(&from) {
                if let Some(Object::Human { id, state }) = &mut tile.object {
                    let id = id.clone();

                    match state {
                        HumanState::Idle { .. } => {
                            let mut outgoing = packet::Outgoing::Arrive { id, x: from.0, z: from.1 }.serialize();

                            for (key, conn) in self.connections.iter() {
                                if let Err(e) = conn.try_write_one(&mut outgoing) {
                                    eprintln!("{e}");
            
                                    Self::schedule_drop(&mut self.schedule_queue, *key);
                                }
                            }
                        },
                        HumanState::Move { direction, moved_at } => {
                            let direction = direction.clone();

                            moved_at.replace(time::Instant::now());

                            if let Some(next) = self.handle_move(id, from, direction, tick) {
                                let job = Job::KeepMove { from: next, tick };
        
                                self.schedule_queue.push(Schedule::new(job, Instant::now() + tick));
                            }
                        },
                    }
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
                self.pointed_at = Some(time::Instant::now());

                if let Some(Object::Human { id, state }) = &mut tile.object { 
                    let id = id.clone();

                    if direction == 0 {
                        *state = HumanState::Idle { moved_at: *match state {
                            HumanState::Idle { moved_at } => moved_at,
                            HumanState::Move { moved_at, .. } => moved_at,
                        }};
                    } else {
                        if let HumanState::Move { direction: old_direction, moved_at } = state {
                            if *old_direction != direction {
                                *state = HumanState::Move { direction, moved_at: *moved_at };
                            }

                            return Ok(());
                        };

                        let tick = time::Duration::from_millis(300);

                        let now = time::Instant::now();

                        if let Some(moved_at) = match state {
                            HumanState::Idle { moved_at } => moved_at,
                            HumanState::Move { moved_at, .. } => moved_at,
                        } {
                            if now < moved_at.to_owned() + tick {
                                return Ok(())
                            }
                        }

                        *state = HumanState::Move { direction, moved_at: Some(now) };

                        if let Some(next) = self.handle_move(id, current, direction, tick) {
                            let job = Job::KeepMove { from: next, tick };
    
                            self.schedule_queue.push(Schedule::new(job, Instant::now() + tick));
                        }
                    }
                }
            },
            _ => {}
        };

        Ok(())
    }

    fn handle_move(&mut self, id: i32, current: (i32, i32), direction: u8,  tick: time::Duration) -> Option<(i32, i32)> {
        let (x, z) = current;
        
        let next = match direction {
            1 => (x, z + 1),
            2 => (x, z - 1),
            3 => (x - 1, z),
            4 => (x + 1, z),
            _ => return None
        };

        let is_unmovable = if let Some(tile) = self.map.get(&next) {
            tile.object.is_some()
        } else {
            true
        };

        if is_unmovable {
            return None;
        }

        self.map.get_mut(&next).unwrap().object = if let Some(tile) = self.map.get_mut(&current) {
            tile.object.take()
        } else {
            return None;
        };

        if let Some(conn) = self.connections.remove(&current) {
            self.connections.insert(next, conn);
        }

        let mut outgoing = packet::Outgoing::Move { id, x: next.0, z: next.1, tick: i64::try_from(tick.as_millis()).unwrap() }.serialize();

        for (key, conn) in self.connections.iter() {
            if let Err(e) = conn.try_write_one(&mut outgoing) {
                eprintln!("{e}");

                Self::schedule_drop(&mut self.schedule_queue, *key);

                continue;
            }
        }

        println!("{:?}", self.pointed_at.take().map(|at| at.elapsed()) );

        return Some(next);
    }

    fn schedule_drop(schedule_queue: &mut BinaryHeap<Schedule>, key: (i32, i32)) {
        let job = Job::Drop(key);

        let schedule = Schedule::now(job);

        schedule_queue.push(schedule);
    }
}
