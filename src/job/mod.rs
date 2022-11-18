use tokio::{time,  net::TcpStream};

mod schedule;

pub use schedule::Schedule;

pub enum Job {
    Sleep(time::Duration),
    Accept(TcpStream),
    Readable((i32, i32)),
}
