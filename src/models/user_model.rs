use crate::models::score_model::ScoreEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub password: String,
    pub completed_tests: u32,
    pub high_scores: HashMap<String, ScoreEntry>,
}
