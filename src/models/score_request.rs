use crate::models::user::ScoreEntry;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ScoreUpdateRequest {
    pub score: ScoreEntry,
    pub timer_duration: u32,
    pub test_completed: u32,
}

#[derive(Deserialize)]
pub struct TimerDurationQuery {
    pub timer_duration: u32,
}
