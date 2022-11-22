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
            1 => {
                if body.len() < 8 {
                    return Err(format!("buffer too short to deserialize, {buf:?}").into())
                }

                Ok(Self::Ping { timestamp: i64::from_le_bytes(body.clone_into_array()) })
            },
            2 => {
                if body.len() < 1 {
                    return Err(format!("buffer too short to deserialize, {buf:?}").into())
                }

                if body.len() > 25 {
                    return Err(format!("buffer too long to deserialize, {buf:?}").into())
                }

                Ok(Incoming::Hello { name: String::from_utf8_lossy(body).into_owned() })
            },
            3 => {
                if body.len() < 1 {
                    return Err(format!("buffer too short to deserialize, {buf:?}").into())
                }

                match body[0] {
                    0 => Ok(Self::Move { direction: 0 }),
                    1 => Ok(Self::Move { direction: 1 }),
                    2 => Ok(Self::Move { direction: 2 }),
                    3 => Ok(Self::Move { direction: 3 }),
                    4 => Ok(Self::Move { direction: 4 }),
                    _ => Err("unexpected arguments".into())
                }
            },
            n => Err(format!("unexpected packet arrived, {n:?}").into())
        }
    }
}
