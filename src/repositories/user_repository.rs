pub use crate::models::api_response::ApiResponse;
pub use crate::models::leaderboard::{
    GetLeaderboardStatsRequest, LeaderboardEntry, LeaderboardResponse,
};
pub use crate::models::score_request::{ScoreUpdateRequest, TimerDurationQuery};
use crate::models::user::{ScoreEntry, User};
use actix_web::HttpResponse;
use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::bson::Document;
use mongodb::bson::{doc, from_bson, to_bson, Bson};
use serde_json::json;
use std::collections::HashMap;

// Convert BSON Document to LeaderboardEntry
pub fn extract_leaderboard_entry(
    doc: &Document,
) -> Result<LeaderboardEntry, mongodb::error::Error> {
    let _id = doc.get_object_id("_id").unwrap_or_default().to_string();
    let username = doc.get_str("username").unwrap_or_default().to_string();
    let completed_tests = doc.get_i32("completed_tests").unwrap_or_default() as u32;

    let high_scores = extract_high_scores(doc)?;

    Ok(LeaderboardEntry {
        _id,
        username,
        completed_tests,
        high_scores,
    })
}

// Helper function to extract high scores from BSON Document
fn extract_high_scores(
    doc: &Document,
) -> Result<HashMap<String, ScoreEntry>, mongodb::error::Error> {
    match doc.get("high_scores") {
        Some(Bson::Document(doc)) => {
            let score_entry: HashMap<String, ScoreEntry> = from_bson(Bson::Document(doc.clone()))?;
            Ok(score_entry)
        }
        _ => Ok(HashMap::new()),
    }
}

// Fetch users with high scores for the specified timer duration
pub async fn fetch_filtered_users(
    collection: &mongodb::Collection<Document>,
    timer_duration: &str,
) -> Result<Vec<LeaderboardEntry>, mongodb::error::Error> {
    let pipeline = create_aggregation_pipeline(timer_duration);

    let mut cursor = collection.aggregate(pipeline).await?;
    let mut users = Vec::new();

    while let Some(doc) = cursor.try_next().await? {
        match extract_leaderboard_entry(&doc) {
            Ok(entry) => users.push(entry),
            Err(e) => eprintln!("Error processing document: {:?}", e),
        }
    }

    log_leaderboard_stats(&users);

    Ok(users)
}

// Create aggregation pipeline for MongoDB query
fn create_aggregation_pipeline(timer_duration: &str) -> Vec<Document> {
    vec![
        doc! { "$match": { format!("high_scores.{}", timer_duration): { "$exists": true } } },
        doc! { "$project": { "_id": 1, "username": 1, "completed_tests": 1, format!("high_scores.{}", timer_duration): 1 } },
    ]
}

// Log leaderboard statistics for debugging
fn log_leaderboard_stats(users: &[LeaderboardEntry]) {
    println!(
        "Leaderboard Stats: {}",
        serde_json::to_string_pretty(&json!(users)).unwrap()
    );
}

// Fetch user by username and return a formatted response
pub async fn fetch_user_and_handle_response(
    collection: &mongodb::Collection<Document>,
    username: &str,
) -> Result<User, HttpResponse> {
    match fetch_user_by_username(collection, username).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(create_not_found_response(username)),
        Err(_) => Err(create_internal_server_error_response(username)),
    }
}

// Create a NotFound response
fn create_not_found_response(username: &str) -> HttpResponse {
    HttpResponse::NotFound().json(ApiResponse {
        status: "error".to_string(),
        message: format!("User '{}' not found in the database", username),
    })
}

// Create an InternalServerError response
fn create_internal_server_error_response(username: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(ApiResponse {
        status: "error".to_string(),
        message: format!(
            "Error fetching user '{}'. Please try again later.",
            username
        ),
    })
}

// Update user scores and handle response
pub async fn update_user_and_handle_response(
    collection: &mongodb::Collection<Document>,
    username: &str,
    user: &User,
) -> HttpResponse {
    match update_user_in_db(collection, username, user).await {
        Ok(result) if result.matched_count > 0 => create_success_response(username, user),
        Ok(_) => create_not_found_update_response(username),
        Err(_) => create_internal_server_error_update_response(username),
    }
}

// Create a success response for updating user
fn create_success_response(username: &str, user: &User) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse {
        status: "success".to_string(),
        message: format!(
            "Scores updated successfully for user '{}'. Timer duration: {} entries",
            username,
            user.high_scores.len()
        ),
    })
}

// Create a NotFound response for updating user
fn create_not_found_update_response(username: &str) -> HttpResponse {
    HttpResponse::NotFound().json(ApiResponse {
        status: "error".to_string(),
        message: format!("User '{}' not found when attempting to update", username),
    })
}

// Create an InternalServerError response for updating user
fn create_internal_server_error_update_response(username: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(ApiResponse {
        status: "error".to_string(),
        message: format!(
            "Failed to update scores for user '{}'. Please try again later.",
            username
        ),
    })
}

// Fetch user by username
pub async fn fetch_user_by_username(
    collection: &mongodb::Collection<Document>,
    username: &str,
) -> Result<Option<User>, mongodb::error::Error> {
    let filter = doc! { "username": username };
    let user_doc = collection.find_one(filter).await?;

    if let Some(doc) = user_doc {
        Ok(Some(from_bson(Bson::Document(doc))?))
    } else {
        Ok(None)
    }
}

// Update user's high scores based on the new score update request
pub fn update_user_high_scores(user: &mut User, score_update: ScoreUpdateRequest) {
    let new_entry = create_score_entry(&score_update);

    let timer_duration_str = score_update.timer_duration.to_string();

    user.high_scores
        .entry(timer_duration_str)
        .and_modify(|existing_entry| {
            if new_entry.wpm > existing_entry.wpm {
                *existing_entry = new_entry.clone();
            }
        })
        .or_insert(new_entry);

    user.completed_tests += 1;
}

// Create a new ScoreEntry
fn create_score_entry(score_update: &ScoreUpdateRequest) -> ScoreEntry {
    ScoreEntry {
        wpm: score_update.score.wpm,
        raw_wpm: score_update.score.raw_wpm,
        accuracy: score_update.score.accuracy,
        date: Utc::now(),
    }
}

// Update user data in the database
pub async fn update_user_in_db(
    collection: &mongodb::Collection<Document>,
    username: &str,
    user: &User,
) -> Result<mongodb::results::UpdateResult, mongodb::error::Error> {
    let filter = doc! { "username": username };
    let update = doc! {
        "$set": {
            "high_scores": to_bson(&user.high_scores).unwrap(),
            "completed_tests": user.completed_tests,
        }
    };
    collection.update_one(filter, update).await
}
