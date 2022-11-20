use tokio::{time,  net::TcpStream};

mod schedule;

pub use schedule::Schedule;

use crate::net::Conn;

pub enum Job {
    Sleep(time::Duration),
    Accept(TcpStream),
    Read((i32, i32)),
    Drop((i32, i32)),
    Arrived(Conn),
    KeepMove { from: (i32, i32), tick: time::Duration },
}
