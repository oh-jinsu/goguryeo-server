use std::io;

use tokio::net::TcpStream;

pub trait Reader {
    fn try_read_one(&self, buf: &mut Vec<u8>) -> io::Result<()>;

    fn try_read_to_end(&self, buf: &mut [u8]) -> io::Result<()>;
}

pub trait Writer {
    fn try_write_one(&self, buf: &mut Vec<u8>) -> io::Result<()>;

    fn try_write_to_end(&self, buf: &mut [u8]) -> io::Result<()>;
}

impl Reader for TcpStream {
    fn try_read_one(&self, buf: &mut Vec<u8>) -> io::Result<()> {
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

        self.try_read_to_end(buf)?;

        Ok(())
    }
    
    fn try_read_to_end(&self, buf: &mut [u8]) -> io::Result<()> {
        let mut pos = 0;

        while pos < buf.len() {
            match self.try_read(&mut buf[pos..]) {
                Ok(0) => return Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
                Ok(n) => { pos += n; },
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

impl Writer for TcpStream {
    fn try_write_one(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        let size: u16 = match buf.len().try_into() {
            Ok(size) => size,
            Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "buffer too large"))
        };

        let mut buf = [&u16::to_le_bytes(size) as &[u8], buf].concat();

        self.try_write_to_end(&mut buf)
    }

    fn try_write_to_end(&self, buf: &mut [u8]) -> io::Result<()> {
        let mut pos = 0;

        while pos < buf.len() {
            match self.try_write(&mut buf[pos..]) {
                Ok(0) => return Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
                Ok(n) => { pos += n; },
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}