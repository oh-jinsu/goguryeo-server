use tokio::time;

use crate::{net::Conn, common::math::Vector3};

pub enum Job {
    Read(Vector3),
    Drop(Vector3),
    Welcome(Conn),
    Move { from: Vector3, tick: time::Duration },
}
