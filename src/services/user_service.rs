use crate::models::user::{
    default_user, DifficultyScores, HighScores, LanguageScores, Score, User,
};
use crate::structs::api_response::ApiResponse;
use crate::structs::leaderboard::ScoreUpdateRequest;
use crate::structs::recaptcha_response::RecaptchaResponse;
use actix_web::HttpResponse;
use chrono::Utc;
use mongodb::bson::{doc, from_bson, to_bson, Bson, Document};
use mongodb::error::Error;
use mongodb::Collection;
use reqwest::Error as ReqwestError;
use std::collections::HashMap;

pub fn create_user(username: String, password: String) -> User {
    let mut user = default_user();
    user.username = username;
    user.password = password;
    user
}

pub async fn insert_user(collection: &Collection<User>, user: User) -> Result<(), String> {
    collection
        .insert_one(user)
        .await
        .map_err(|_| "Error adding user".to_string())?;
    Ok(())
}

pub async fn verify_recaptcha(token: &str) -> Result<RecaptchaResponse, ReqwestError> {
    let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY must be set");

    let client = reqwest::Client::new();
    let response = client
        .post("https://www.google.com/recaptcha/api/siteverify")
        .form(&[("secret", secret_key), ("response", token.to_string())])
        .send()
        .await?;

    let recaptcha_response = response.json::<RecaptchaResponse>().await?;

    Ok(recaptcha_response)
}

pub async fn is_user_exists(collection: &Collection<User>, username: &str) -> Result<bool, Error> {
    let filter = doc! { "username": username };
    match collection.find_one(filter).await {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => Err(e),
    }
}

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
        Some(scores) => scores.languages.len(),
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

    let timer_duration_str = score_update.duration.clone(); // Use duration as String
    let language = score_update.language.clone();
    let difficulty = score_update.difficulty.clone();

    if let Some(high_scores) = &mut user.high_scores {
        let language_scores = high_scores
            .languages
            .entry(language.clone())
            .or_insert_with(|| LanguageScores {
                difficulties: HashMap::new(),
            });

        let difficulty_scores = language_scores
            .difficulties
            .entry(difficulty.clone())
            .or_insert_with(|| DifficultyScores {
                scores: HashMap::new(),
            });

        let existing_score = difficulty_scores
            .scores
            .entry(timer_duration_str.clone())
            .or_insert_with(|| Score {
                wpm: 0,
                raw_wpm: 0,
                accuracy: 0.0,
                date: Utc::now(),
            });

        if new_entry.wpm > existing_score.wpm {
            *existing_score = new_entry;
        }
    } else {
        // Initialize high_scores if it does not exist
        user.high_scores = Some(HighScores {
            languages: {
                let mut map = HashMap::new();
                map.insert(
                    language.clone(),
                    LanguageScores {
                        difficulties: {
                            let mut diff_map = HashMap::new();
                            diff_map.insert(
                                difficulty.clone(),
                                DifficultyScores {
                                    scores: {
                                        let mut score_map = HashMap::new();
                                        score_map.insert(timer_duration_str.clone(), new_entry);
                                        score_map
                                    },
                                },
                            );
                            diff_map
                        },
                    },
                );
                map
            },
        });
    }

    if let Some(completed_tests) = &mut user.completed_tests {
        *completed_tests += 1;
    }
}

fn create_score_entry(score_update: &ScoreUpdateRequest) -> Score {
    Score {
        wpm: score_update.score.wpm,
        raw_wpm: score_update.score.raw_wpm,
        accuracy: score_update.score.accuracy,
        date: score_update.score.date,
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
