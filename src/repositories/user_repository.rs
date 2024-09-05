use crate::models::score_model::ScoreEntry;
use crate::models::user_model::User;
use crate::routes::score_routes::ScoreUpdateRequest;
use actix_web::HttpResponse;
use chrono::Utc;
use mongodb::bson::Document;
use mongodb::bson::{doc, from_bson, to_bson, Bson};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse {
    status: String,
    message: String,
}

pub async fn fetch_user_and_handle_response(
    collection: &mongodb::Collection<Document>,
    username: &str,
) -> Result<User, HttpResponse> {
    match fetch_user_by_username(collection, username).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(HttpResponse::NotFound().json(ApiResponse {
            status: "error".to_string(),
            message: format!("User '{}' not found in the database", username),
        })),
        Err(_) => Err(HttpResponse::InternalServerError().json(ApiResponse {
            status: "error".to_string(),
            message: format!(
                "Error fetching user '{}'. Please try again later.",
                username
            ),
        })),
    }
}

pub async fn update_user_and_handle_respond(
    collection: &mongodb::Collection<Document>,
    username: &str,
    user: &User,
) -> HttpResponse {
    match update_user_in_db(collection, username, user).await {
        Ok(result) if result.matched_count > 0 => HttpResponse::Ok().json(ApiResponse {
            status: "success".to_string(),
            message: format!(
                "Scores updated successfully for user '{}'. Timer duration: {} seconds",
                username,
                user.high_scores.len() // Example usage of high_scores length
            ),
        }),
        Ok(_) => HttpResponse::NotFound().json(ApiResponse {
            status: "error".to_string(),
            message: format!("User '{}' not found when attempting to update", username),
        }),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse {
            status: "error".to_string(),
            message: format!(
                "Failed to update scores for user '{}'. Please try again later.",
                username
            ),
        }),
    }
}

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

pub fn update_user_high_scores(user: &mut User, score_update: ScoreUpdateRequest) {
    let new_entry = ScoreEntry {
        wpm: score_update.score.wpm,
        raw_wpm: score_update.score.raw_wpm,
        accuracy: score_update.score.accuracy,
        date: Utc::now(),
    };

    let timer_duration_str = score_update.timer_duration.to_string();

    user.high_scores
        .entry(timer_duration_str)
        .and_modify(|existing_entry| {
            if new_entry.wpm > existing_entry.wpm {
                *existing_entry = new_entry.clone();
            }
        })
        .or_insert(new_entry);

    if score_update.test_completed >= 100 {
        user.completed_tests += 1;
    }
}

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
