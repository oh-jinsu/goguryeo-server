mod job;

use std::net::SocketAddr;
use std::{error::Error, io};
use std::collections::HashMap;

use futures::future::select_all;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use crate::{common::AutoIncrement, net::{Conn, packet}};

use self::job::Job;

pub struct Gatekeeper {
    auto_increment: AutoIncrement,
    listener: TcpListener,
    connections: HashMap<SocketAddr, Conn>,
    sender: mpsc::Sender<Conn>
}

impl Gatekeeper {
    pub fn new(listener: TcpListener, tx: mpsc::Sender<Conn>) -> Self {
        Gatekeeper {
            auto_increment: AutoIncrement::new(),
            listener,
            connections: HashMap::new(),
            sender: tx,
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
            if let Ok(conn) = self.listener.accept().await {
                Some(Job::Accept(conn))
            } else {
                None
            }
        } else {
            Some(tokio::select! {
                Ok(conn) = self.listener.accept() => {
                    Job::Accept(conn)
                },
                (Ok(addr), _, _) = select_all(self.connections.iter_mut().map(|(key, conn)| Box::pin(async {
                    conn.readable().await?;

                    Ok::<&SocketAddr, Box<dyn Error>>(key)
                }))) => {
                    Job::Read(addr.clone())
                },
            })
        }
    }

    async fn handle_job(&mut self, job: Job) -> Result<(), Box<dyn Error>> {
        match job {
            Job::Accept((stream, addr)) => {
                let id = self.auto_increment.take();

                let conn = Conn::new(stream, id);
        
                self.connections.insert(addr, conn);
            },
            Job::Read(addr) => if let Some(conn) = self.connections.get(&addr) {
                let mut buf = vec![0 as u8; 2];

                if let Err(e) = conn.try_read_one(&mut buf) {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        eprintln!("{e}");

                        self.connections.remove(&addr);
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

                if let Err(e) = self.handle_packet(packet, addr) {
                    eprintln!("{e}");

                    self.connections.remove(&addr);
                }
            },
        }

        Ok(())
    }

    fn handle_packet(&mut self, packet: packet::Incoming, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
        match packet {
            packet::Incoming::Hello { name } => if let Some(conn) = self.connections.remove(&addr) {
                let outgoing = packet::Outgoing::Hello { id: conn.id, name };

                conn.try_write_one(&mut outgoing.serialize())?;

                let _ = self.sender.try_send(conn);
            },
            packet::Incoming::Ping { timestamp } => if let Some(conn) = self.connections.get(&addr) {
                let outgoing = packet::Outgoing::Pong { timestamp };

                conn.try_write_one(&mut outgoing.serialize())?;
            },
            _ => {}
        };

        Ok(())
    }
}
