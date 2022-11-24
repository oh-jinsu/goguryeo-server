#[derive(Debug)]
pub enum Outgoing {
    Pong { timestamp: i64 },
    Hello { id: [u8; 16] },
    Connect { id: [u8; 16], x: i32, y: i32, z: i32 },
    Disconnect { id: [u8; 16] },
    Introduce { users: Vec<([u8; 16], i32, i32, i32)> },
    Move { id: [u8; 16], x: i32, y: i32, z: i32, tick: i64 },
    Arrive { id: [u8; 16], x: i32, y: i32, z: i32 },
}

impl Outgoing {
    pub fn serialize(self) -> Vec<u8> {
        match self {
            Outgoing::Pong { timestamp } => [
                &[1 as u8, 0] as &[u8],
                &timestamp.to_le_bytes(),
            ].concat(),
            Outgoing::Hello { id } => [
                &[2 as u8, 0] as &[u8],
                &id,
            ].concat(),
            Outgoing::Connect { id, x, y, z } => [
                &[3 as u8, 0] as &[u8],
                &id,
                &x.to_le_bytes(),
                &y.to_le_bytes(),
                &z.to_le_bytes(),
            ].concat(),
            Outgoing::Disconnect { id } => [
                &[4 as u8, 0] as &[u8],
                &id,
            ].concat(),
            Outgoing::Introduce { users } => [
                &[5 as u8, 0] as &[u8],
                &users.iter().flat_map(|(id, x, y, z)| [
                    id as &[u8],
                    &x.to_le_bytes(),
                    &y.to_le_bytes(),
                    &z.to_le_bytes(),
                ].concat()).collect::<Vec<u8>>()
            ].concat(),
            Outgoing::Move { id, x, y, z, tick } => [
                &[6 as u8, 0] as &[u8],
                &id,
                &x.to_le_bytes(),
                &y.to_le_bytes(),
                &z.to_le_bytes(),
                &tick.to_le_bytes(),
            ].concat(),
            Outgoing::Arrive { id, x, y, z } => [
                &[7 as u8, 0] as &[u8],
                &id,
                &x.to_le_bytes(),
                &y.to_le_bytes(),
                &z.to_le_bytes(),
            ].concat(),
        }
    }
}
