use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Score {
    pub wpm: u32,
    pub raw_wpm: u32,
    pub accuracy: f32,
    pub date: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DifficultyScores {
    pub scores: HashMap<String, Score>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LanguageScores {
    pub difficulties: HashMap<String, DifficultyScores>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HighScores {
    #[serde(default = "default_languages")]
    pub languages: HashMap<String, LanguageScores>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub password: String,
    #[serde(default = "default_completed_tests")]
    pub completed_tests: Option<u32>,
    #[serde(default)]
    pub high_scores: Option<HighScores>,
    #[serde(default = "default_created_at")]
    pub created_at: Option<DateTime<Utc>>,
}

// Provide default for completed_tests
fn default_completed_tests() -> Option<u32> {
    Some(0)
}

// Provide default for created_at
fn default_created_at() -> Option<DateTime<Utc>> {
    Some(Utc::now())
}

// Provide default for languages to prevent deserialization errors
fn default_languages() -> HashMap<String, LanguageScores> {
    HashMap::new() // Return an empty map as default
}

pub fn default_user() -> User {
    let default_score = Score {
        wpm: 0,
        raw_wpm: 0,
        accuracy: 0.0,
        date: Utc::now(),
    };

    let mut difficulty_scores = HashMap::new();
    for duration in ["15", "30", "60", "90"].iter() {
        difficulty_scores.insert(duration.to_string(), default_score.clone());
    }

    let mut difficulties = HashMap::new();
    for difficulty in ["hard", "normal"].iter() {
        difficulties.insert(
            difficulty.to_string(),
            DifficultyScores {
                scores: difficulty_scores.clone(),
            },
        );
    }

    let mut languages = HashMap::new();
    for language in ["english", "chinese"].iter() {
        languages.insert(
            language.to_string(),
            LanguageScores {
                difficulties: difficulties.clone(),
            },
        );
    }

    User {
        username: "".to_string(),
        password: "".to_string(),
        completed_tests: Some(0),                    // Default to Some(0)
        high_scores: Some(HighScores { languages }), // Ensure high_scores is Some with a valid structure
        created_at: Some(Utc::now()),                // Automatically set to current time
    }
}
