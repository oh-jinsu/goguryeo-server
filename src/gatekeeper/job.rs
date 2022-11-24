use std::net::SocketAddr;

use tokio::net::TcpStream;

pub enum Job {
    Accept((TcpStream, SocketAddr)),
    Read(SocketAddr),
}