use crate::models::user::ScoreEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct LeaderboardEntry {
    #[serde(rename = "_id")]
    pub _id: String,
    pub username: String,
    pub completed_tests: u32,
    pub high_scores: HashMap<String, ScoreEntry>,
}

#[derive(Serialize)]
pub struct LeaderboardResponse {
    pub status: String,
    pub message: String,
    pub leaderboard: Vec<LeaderboardEntry>,
}

#[derive(Serialize)]
pub struct GetLeaderboardStatsRequest {
    pub timer_duration: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ScoreUpdateRequest {
    pub score: ScoreEntry,
    pub timer_duration: u32,
    pub test_completed: u32,
}

#[derive(Deserialize)]
pub struct TimerDurationQuery {
    pub timer_duration: String,
    pub page: String,
    pub limit: String,
}
