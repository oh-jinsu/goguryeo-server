use tokio::{time, net::TcpStream};

use crate::common::math::Vector3;

pub enum Job {
    Read(Vector3),
    Drop(Vector3),
    Welcome(TcpStream, [u8; 16]),
    Move { from: Vector3, tick: time::Duration },
}
