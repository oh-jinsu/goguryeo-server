use tokio::time;

use crate::net::Conn;

pub enum Job {
    Read((i32, i32)),
    Drop((i32, i32)),
    Welcome(Conn),
    Move { from: (i32, i32), tick: time::Duration },
}
