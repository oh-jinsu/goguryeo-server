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
