use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::time::Instant;

pub const PENDING: char = ' ';
pub const COMPLETED: char = '✓';

#[derive(Serialize, Deserialize, Debug, Eq, Clone)]
pub struct Task {
    #[serde(with = "serde_millis")]
    pub time: Instant,
    pub description: String,
    pub status: char,
}

impl Task {
    pub fn new(description: String) -> Task {
        return Task {
            time: Instant::now(),
            description: description,
            status: PENDING,
        };
    }

    pub fn complete(&mut self) {
        self.status = COMPLETED;
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
