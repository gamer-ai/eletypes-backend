use crate::models::user::{HighScores, Score};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LeaderboardEntry {
    #[serde(rename = "_id")]
    pub _id: String,
    pub username: String,
    pub completed_tests: u32,
    pub high_scores: HighScores,
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

#[derive(Deserialize, Debug)]
pub struct ScoreUpdateRequest {
    pub duration: String,
    pub language: String,
    pub difficulty: String,
    pub score: Score,
}

#[derive(Deserialize)]
pub struct GetLeaderboardStatsQueries {
    pub timer_duration: String,
    pub page: String,
    pub limit: String,
    pub difficulty: String,
    pub language: String,
}
