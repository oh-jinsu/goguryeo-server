use tokio::{time, net::TcpStream};

use crate::common::math::Vector3;

pub enum Job {
    Read([u8; 16]),
    Drop([u8; 16]),
    Welcome(TcpStream, [u8; 16]),
    Move { from: Vector3, tick: time::Duration },
}
