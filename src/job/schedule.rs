use tokio::time;

use super::Job;

pub struct Schedule {
    pub job: Job,
    pub deadline: time::Instant,
}

impl Schedule {
    pub fn new(job: Job, deadline: time::Instant) -> Self {
        Schedule { job, deadline }
    }

    pub fn now(job: Job) -> Self {
        Schedule { job, deadline: time::Instant::now() }
    }
}

impl PartialEq for Schedule {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline
    }
}

impl Eq for Schedule {}

impl PartialOrd for Schedule {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.deadline.partial_cmp(&other.deadline).map(|ordering| ordering.reverse())
    }
}

impl Ord for Schedule {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deadline.cmp(&other.deadline).reverse()
    }
}
