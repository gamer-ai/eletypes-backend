use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ScoreEntry {
    pub wpm: u32,
    pub raw_wpm: u32,
    pub accuracy: f32,
    pub date: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub password: String,
    pub completed_tests: u32,
    pub high_scores: HashMap<String, ScoreEntry>,
}
