use std::io;

use tokio::net::TcpStream;

pub struct Conn {
    pub id: usize,
    stream: TcpStream,
}

impl Conn {
    pub fn new(stream: TcpStream, id: usize) -> Conn {
        println!("connection {id} accepted");

        Conn { stream, id }
    }

    pub async fn readable(&self) -> io::Result<()> {
        self.stream.readable().await
    }

    pub fn try_read_one(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        if buf.len() < 2 {
            buf.resize(2, 0);
        }

        self.try_read(&mut buf[..2])?;

        let size = usize::from(u16::from_le_bytes([buf[0], buf[1]]));

        if size == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, format!("invalid size, {size}")))
        }

        if size > 8096 {
            return Err(io::Error::new(io::ErrorKind::Other, format!("packet too large, {size}")))
        }

        buf.resize(size, 0);

        self.try_read(buf)?;

        Ok(())
    }
    
    fn try_read(&self, buf: &mut [u8]) -> io::Result<()> {
        let mut pos = 0;

        while pos < buf.len() {
            match self.stream.try_read(&mut buf[pos..]) {
                Ok(0) => return Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
                Ok(n) => { pos += n; },
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    pub fn try_write_one(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        let size: u16 = match buf.len().try_into() {
            Ok(size) => size,
            Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "buffer too large"))
        };

        let mut buf = [&u16::to_le_bytes(size) as &[u8], buf].concat();

        self.try_write(&mut buf)
    }

    fn try_write(&self, buf: &mut [u8]) -> io::Result<()> {
        let mut pos = 0;

        while pos < buf.len() {
            match self.stream.try_write(&mut buf[pos..]) {
                Ok(0) => return Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
                Ok(n) => { pos += n; },
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        println!("connection {} dropped", self.id);
    }
}
