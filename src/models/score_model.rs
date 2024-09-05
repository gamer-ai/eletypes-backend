use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ScoreEntry {
    pub wpm: u32,
    pub raw_wpm: u32,
    pub accuracy: f32,
    pub date: DateTime<Utc>,
}
