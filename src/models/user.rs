use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScoreEntry {
    pub wpm: u32,
    pub raw_wpm: u32,
    pub accuracy: f32,
    pub date: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub password: String,
    pub completed_tests: Option<u32>,
    pub high_scores: Option<HashMap<String, ScoreEntry>>,
}

pub fn default_user() -> User {
    let mut scores = HashMap::new();
    for duration in ["15", "30", "60", "90"].iter() {
        scores.insert(
            duration.to_string(),
            ScoreEntry {
                wpm: 0,
                raw_wpm: 0,
                accuracy: 0.0,
                date: Utc::now(),
            },
        );
    }

    User {
        username: "".to_string(),
        password: "".to_string(),
        completed_tests: Some(0),
        high_scores: Some(scores),
    }
}
