use crate::models::api_response::ApiResponse;
use crate::models::score_request::ScoreUpdateRequest;
use crate::models::user::{ScoreEntry, User};
use actix_web::HttpResponse;
use chrono::Utc;
use mongodb::bson::{doc, from_bson, to_bson, Bson, Document};
use mongodb::Collection;

pub async fn fetch_user_and_handle_response(
    collection: &Collection<Document>,
    username: &str,
) -> Result<User, HttpResponse> {
    match fetch_user_by_username(collection, username).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(create_not_found_response(username)),
        Err(_) => Err(create_internal_server_error_response(username)),
    }
}

fn create_not_found_response(username: &str) -> HttpResponse {
    HttpResponse::NotFound().json(ApiResponse {
        status: "error".to_string(),
        message: format!("User '{}' not found in the database", username),
    })
}

fn create_internal_server_error_response(username: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(ApiResponse {
        status: "error".to_string(),
        message: format!(
            "Error fetching user '{}'. Please try again later.",
            username
        ),
    })
}

pub async fn save_user_scores(
    collection: &Collection<Document>,
    username: &str,
    user: &User,
) -> HttpResponse {
    match update_user_in_db(collection, username, user).await {
        Ok(result) if result.matched_count > 0 => create_success_response(username, user),
        Ok(_) => create_not_found_update_response(username),
        Err(_) => create_internal_server_error_update_response(username),
    }
}

fn create_success_response(username: &str, user: &User) -> HttpResponse {
    let high_scores_len = match &user.high_scores {
        Some(scores) => scores.len(),
        None => 0,
    };

    HttpResponse::Ok().json(ApiResponse {
        status: "success".to_string(),
        message: format!(
            "Scores updated successfully for user '{}'. Timer duration: {} entries",
            username, high_scores_len
        ),
    })
}

fn create_not_found_update_response(username: &str) -> HttpResponse {
    HttpResponse::NotFound().json(ApiResponse {
        status: "error".to_string(),
        message: format!("User '{}' not found when attempting to update", username),
    })
}

fn create_internal_server_error_update_response(username: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(ApiResponse {
        status: "error".to_string(),
        message: format!(
            "Failed to update scores for user '{}'. Please try again later.",
            username
        ),
    })
}

pub fn update_user_high_scores(user: &mut User, score_update: ScoreUpdateRequest) {
    let new_entry = create_score_entry(&score_update);

    let timer_duration_str = score_update.timer_duration.to_string();

    if let Some(high_scores) = &mut user.high_scores {
        high_scores
            .entry(timer_duration_str)
            .and_modify(|existing_entry| {
                if new_entry.wpm > existing_entry.wpm {
                    *existing_entry = new_entry.clone();
                }
            })
            .or_insert(new_entry);
    }

    if let Some(completed_tests) = &mut user.completed_tests {
        *completed_tests += 1;
    }
}

fn create_score_entry(score_update: &ScoreUpdateRequest) -> ScoreEntry {
    ScoreEntry {
        wpm: score_update.score.wpm,
        raw_wpm: score_update.score.raw_wpm,
        accuracy: score_update.score.accuracy,
        date: Utc::now(),
    }
}

pub async fn update_user_in_db(
    collection: &Collection<Document>,
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

pub async fn fetch_user_by_username(
    collection: &Collection<Document>,
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
