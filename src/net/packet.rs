use std::error::Error;

use crate::common::Bytes;

#[derive(Debug)]
pub enum Incoming {
    Hello { name: String },
    Ping { timestamp: i64 },
    Move { direction: u8 }
}

impl Incoming {
    pub fn deserialize(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        if buf.len() < 2 {
            return Err(format!("buffer too short to deserialize, {buf:?}").into())
        }

        let serial = u16::from_le_bytes([buf[0], buf[1]]);

        let body = &buf[2..];

        match serial {
            1 => Ok(Incoming::Hello { name: String::from_utf8_lossy(body.truncate_last()).into_owned() }),
            3 => Ok(Self::Ping { timestamp: i64::from_le_bytes(body.clone_into_array()) }),
            4 => match body[0] {
                0 => Ok(Self::Move { direction: 0 }),
                1 => Ok(Self::Move { direction: 1 }),
                2 => Ok(Self::Move { direction: 2 }),
                3 => Ok(Self::Move { direction: 3 }),
                4 => Ok(Self::Move { direction: 4 }),
                _ => Err("unexpected arguments".into())
            },
            n => Err(format!("unexpected packet arrived, {n:?}").into())
        }
    }
}

#[derive(Debug)]
pub enum Outgoing {
    Welcome { id: usize, name: String },
    Pong { timestamp: i64 },
    Move { id: usize, x: i32, y: i32 },
}

impl Outgoing {
    pub fn serialize(self) -> Vec<u8> {
        match self {
            Outgoing::Welcome { id, name } => [
                &(1 as u16).to_le_bytes() as &[u8],
                &id.to_le_bytes(),
                &name.as_bytes().to_sized(25),
            ].concat(),
            Outgoing::Pong { timestamp } => [
                &(3 as u16).to_le_bytes() as &[u8],
                &timestamp.to_le_bytes(),
            ].concat(),
            Outgoing::Move { id, x, y } => [
                &(4 as u16).to_le_bytes() as &[u8],
                &id.to_le_bytes(),
                &x.to_le_bytes(),
                &y.to_le_bytes(),
            ].concat(),
        }
    }
}
