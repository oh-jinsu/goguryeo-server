use tokio::time;

pub enum Object {
    Human {
        id: [u8; 16],
        state: HumanState,
    },
}

impl Object {
    pub fn new_human(id: [u8; 16]) -> Self {
        Object::Human { id, state: HumanState::Idle { updated_at: None } }
    }
}

pub enum HumanState {
    Idle { updated_at: Option<time::Instant> },
    Move { direction: u8, updated_at: Option<time::Instant> },
}