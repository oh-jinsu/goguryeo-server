use std::{error::Error, collections::{BinaryHeap, HashMap}, io};

use futures::future::select_all;
use mmorpg::{job::{Schedule, Job}, net::{packet, Conn}, common::AutoIncrement};
use tokio::{net::TcpListener, time};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut key = AutoIncrement::new();

    let mut schedule_queue: BinaryHeap<Schedule> = BinaryHeap::new();

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let mut connections: HashMap<usize, Conn> = HashMap::new();

    loop {
        if schedule_queue.is_empty() {
            let schedule = Schedule::new(
                Job::Sleep(time::Duration::ZERO),
                time::Instant::now() + time::Duration::from_secs(1),
            );
    
            schedule_queue.push(schedule);
        }

        let job = if connections.is_empty() {
            if let Ok((stream, _)) = listener.accept().await {
                Job::Accept(stream)
            } else {
                continue;
            }
        } else {
            tokio::select! {
                _ = time::sleep_until(schedule_queue.peek().unwrap().deadline) => {
                    schedule_queue.pop().unwrap().job
                },
                Ok((stream, _)) = listener.accept() => {
                    Job::Accept(stream)
                }
                (Ok(key), _, _) = select_all(connections.iter_mut().map(|(key, conn)| Box::pin(async {
                    conn.readable().await?;

                    Ok::<&usize, Box<dyn Error>>(key)
                }))) => {
                    Job::Readable(key.clone())
                }
            }
        };

        match job {
            Job::Sleep(duration) => time::sleep(duration).await,
            Job::Accept(stream) => {
                let key = key.take();

                let conn = Conn::new(stream);

                connections.insert(key, conn);
            },
            Job::Readable(key) => {
                let timestamp = time::Instant::now();

                let conn = match connections.get(&key) {
                    Some(stream) => stream,
                    None => continue,
                };

                let mut buf = vec![0 as u8; 2];

                if let Err(e) = conn.try_read_line(&mut buf) {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        eprintln!("{e}");

                        connections.remove(&key);
                    }

                    continue;
                }

                let packet = match packet::Incoming::deserialize(&mut buf) {
                    Ok(packet) => packet,
                    Err(e) => {
                        eprintln!("{e}");

                        continue;
                    }
                };

                match packet {
                    packet::Incoming::Hello { name } => {
                        let outgoing = packet::Outgoing::Welcome { name };

                        if let Err(e) = conn.try_write_line(&mut outgoing.serialize()) {
                            if e.kind() != io::ErrorKind::WouldBlock {
                                eprintln!("{e}");
        
                                connections.remove(&key);
                            }
        
                            continue;
                        }
                    },
                    packet::Incoming::Ping { timestamp } => {
                        let outgoing = packet::Outgoing::Pong { timestamp };

                        if let Err(e) = conn.try_write_line(&mut outgoing.serialize()) {
                            if e.kind() != io::ErrorKind::WouldBlock {
                                eprintln!("{e}");
        
                                connections.remove(&key);
                            }
        
                            continue;
                        }
                    },
                    packet::Incoming::Move { direction } => {
                        
                    },
                };

                println!("incoming handling spent {:?}", timestamp.elapsed());
            },
        }
    }
}
