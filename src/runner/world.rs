use std::error::Error;
use std::collections::{BinaryHeap, HashMap};
use std::io;

use futures::future::select_all;
use tokio::net::TcpListener;
use tokio::time;

use crate::job::Job;
use crate::net::packet;
use crate::{common::AutoIncrement, job::Schedule, net::Conn};

pub struct World {
    key: AutoIncrement,
    listener: TcpListener,
    schedule_queue: BinaryHeap<Schedule>,
    connections: HashMap<usize, Conn>,
}

impl World {
    pub fn new(listener: TcpListener) -> Self {
        let key = AutoIncrement::new();

        World {
            key,
            listener,
            schedule_queue: BinaryHeap::new(),
            connections: HashMap::new(),
        }
    }
    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            self.ensure_schedule();
    
            if let Some(job) = self.select_job().await {
                if let Err(e) = self.handle_job(job).await {
                    eprintln!("{e}");
                }
            }
        }
    }

    fn ensure_schedule(&mut self) {
        if self.schedule_queue.is_empty() {
            let schedule = Schedule::new(
                Job::Sleep(time::Duration::ZERO),
                time::Instant::now() + time::Duration::from_secs(1),
            );
    
            self.schedule_queue.push(schedule);
        }
    }

    async fn select_job(&mut self) -> Option<Job> {
        if self.connections.is_empty() {
            if let Ok((stream, _)) = self.listener.accept().await {
                Some(Job::Accept(stream))
            } else {
                None
            }
        } else {
            Some(tokio::select! {
                _ = time::sleep_until(self.schedule_queue.peek().unwrap().deadline) => {
                    self.schedule_queue.pop().unwrap().job
                },
                Ok((stream, _)) = self.listener.accept() => {
                    Job::Accept(stream)
                }
                (Ok(key), _, _) = select_all(self.connections.iter_mut().map(|(key, conn)| Box::pin(async {
                    conn.readable().await?;

                    Ok::<&usize, Box<dyn Error>>(key)
                }))) => {
                    Job::Readable(key.clone())
                }
            })
        }
    }

    async fn handle_job(&mut self, job: Job) -> Result<(), Box<dyn Error>> {
        match job {
            Job::Sleep(duration) => time::sleep(duration).await,
            Job::Accept(stream) => {
                let key = self.key.take();

                let conn = Conn::new(stream);

                self.connections.insert(key, conn);
            },
            Job::Readable(key) => if let Some(conn) = self.connections.get(&key) {
                let timestamp = time::Instant::now();

                let mut buf = vec![0 as u8; 2];

                if let Err(e) = conn.try_read_line(&mut buf) {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        eprintln!("{e}");

                        self.connections.remove(&key);
                    }

                    return Ok(());
                }

                let packet = match packet::Incoming::deserialize(&mut buf) {
                    Ok(packet) => packet,
                    Err(e) => {
                        eprintln!("{e}");

                        return Ok(());
                    }
                };

                if let Err(e) = self.handle_packet(packet, key) {
                    eprintln!("{e}");

                    self.connections.remove(&key);
                }

                println!("incoming handling spent {:?}", timestamp.elapsed());
            },
        }

        Ok(())
    }

    fn handle_packet(&mut self, packet: packet::Incoming, key: usize) -> Result<(), Box<dyn Error>> {
        match packet {
            packet::Incoming::Hello { name } => {
                let outgoing = packet::Outgoing::Welcome { name };

                if let Some(conn) = self.connections.get(&key) {
                    conn.try_write_line(&mut outgoing.serialize())?;
                };
            },
            packet::Incoming::Ping { timestamp } => {
                let outgoing = packet::Outgoing::Pong { timestamp };

                if let Some(conn) = self.connections.get(&key) {
                    conn.try_write_line(&mut outgoing.serialize())?;
                };
            },
            packet::Incoming::Move { direction } => {
                
            },
        };

        Ok(())
    }
}