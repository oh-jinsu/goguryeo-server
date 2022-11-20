use std::error::Error;

use crate::common::Bytes;

#[derive(Debug)]
pub enum Incoming {
    Ping { timestamp: i64 },
    Hello { name: String },
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
            1 => Ok(Self::Ping { timestamp: i64::from_le_bytes(body.clone_into_array()) }),
            2 => Ok(Incoming::Hello { name: String::from_utf8_lossy(body).into_owned() }),
            3 => match body[0] {
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
    Pong { timestamp: i64 },
    Hello { id: i32, name: String },
    Connect { id: i32, x: i32, z: i32 },
    Disconnect { id: i32 },
    Introduce { users: Vec<(i32, i32, i32)> },
    Move { id: i32, x: i32, z: i32, tick: i64 },
    Arrive { id: i32, x: i32, z: i32 },
}

impl Outgoing {
    pub fn serialize(self) -> Vec<u8> {
        match self {
            Outgoing::Pong { timestamp } => [
                &[1 as u8, 0] as &[u8],
                &timestamp.to_le_bytes(),
            ].concat(),
            Outgoing::Hello { id, name } => [
                &[2 as u8, 0] as &[u8],
                &id.to_le_bytes(),
                &name.as_bytes(),
            ].concat(),
            Outgoing::Connect { id, x, z } => [
                &[3 as u8, 0] as &[u8],
                &id.to_le_bytes(),
                &x.to_le_bytes(),
                &z.to_le_bytes(),
            ].concat(),
            Outgoing::Disconnect { id } => [
                &[4 as u8, 0] as &[u8],
                &id.to_le_bytes(),
            ].concat(),
            Outgoing::Introduce { users } => [
                &[5 as u8, 0] as &[u8],
                &users.iter().flat_map(|(id, x, y)| [
                    &id.to_le_bytes() as &[u8],
                    &x.to_le_bytes(),
                    &y.to_le_bytes(),
                ].concat()).collect::<Vec<u8>>()
            ].concat(),
            Outgoing::Move { id, x, z, tick } => [
                &[6 as u8, 0] as &[u8],
                &id.to_le_bytes(),
                &x.to_le_bytes(),
                &z.to_le_bytes(),
                &tick.to_le_bytes(),
            ].concat(),
            Outgoing::Arrive { id, x, z } => [
                &[7 as u8, 0] as &[u8],
                &id.to_le_bytes(),
                &x.to_le_bytes(),
                &z.to_le_bytes(),
            ].concat(),
        }
    }
}
