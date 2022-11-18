use tokio::{time,  net::TcpStream};

mod schedule;

pub use schedule::Schedule;

use crate::net::Conn;

pub enum Job {
    Sleep(time::Duration),
    Accept(TcpStream),
    Readable((i32, i32)),
    Arrived(Conn),
    Move { current: (i32, i32), tick: time::Duration },
}
