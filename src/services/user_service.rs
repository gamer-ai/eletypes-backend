use crate::models::user::{
    default_user, DifficultyScores, HighScores, LanguageScores, Score, User,
};
use crate::structs::api_response::ApiResponse;
use crate::structs::api_response::{error_response, success_response};
use crate::structs::claims::Claims;
use crate::structs::leaderboard::ScoreUpdateRequest;
use crate::structs::recaptcha_response::RecaptchaResponse;
use actix_web::cookie::time::Duration;
use actix_web::{cookie::Cookie, HttpResponse};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use mongodb::bson::{doc, from_bson, to_bson, to_document, Bson, Document};
use mongodb::error::Error;
use mongodb::Collection;
use reqwest::Error as ReqwestError;
use std::collections::HashMap;
use std::env;

pub async fn process_user_registration(
    collection: &Collection<Document>,
    username: &str,
    password: &str,
) -> HttpResponse {
    match is_user_exists(collection, username).await {
        Ok(true) => HttpResponse::BadRequest().json(error_response("Username already taken.")),
        Ok(false) => {
            let user = create_user(username.to_string(), password.to_string());
            match insert_user(collection, user).await {
                Ok(_) => HttpResponse::Ok().json(success_response("User successfully registered.")),
                Err(err) => HttpResponse::InternalServerError().json(error_response(&err)),
            }
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(error_response("Error checking username availability.")),
    }
}

pub async fn verify_recaptcha_and_check(token: &str) -> Result<(), HttpResponse> {
    match verify_recaptcha(token).await {
        Ok(response) if response.success => Ok(()),
        Ok(_) => Err(HttpResponse::BadRequest().json(error_response(
            "reCAPTCHA verification failed. It seems you are not a human.",
        ))),
        Err(err) => {
            eprintln!("Error verifying reCAPTCHA: {:?}", err);
            Err(HttpResponse::InternalServerError().json(error_response(
                "An error occurred while verifying reCAPTCHA. Please try again later.",
            )))
        }
    }
}

pub fn create_http_only_cookie(token: String) -> Cookie<'static> {
    Cookie::build("user_jwt_token", token)
        .http_only(true)
        .secure(true)
        .path("/")
        .max_age(Duration::new(3600, 0))
        .finish()
}

pub async fn authenticate_user(
    collection: &Collection<Document>,
    username: &str,
    password: &str,
) -> Result<bool, Error> {
    if let Some(user) = fetch_user_by_username(collection, username).await? {
        Ok(user.password == password)
    } else {
        Ok(false)
    }
}

pub fn generate_jwt(username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret_key = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let expiration_time = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize;
    let claims = Claims {
        sub: username.to_string(),
        exp: expiration_time,
    };

    let encoding_key = EncodingKey::from_secret(secret_key.as_bytes());

    let token = encode(&Header::default(), &claims, &encoding_key)?;

    Ok(token)
}

pub fn validate_credentials(
    username: &str,
    token: &str,
    password: &str,
    confirmation_password: Option<&str>,
) -> Option<HttpResponse> {
    if username.is_empty() {
        return Some(HttpResponse::BadRequest().json(error_response("Username cannot be empty.")));
    }
    if token.is_empty() {
        return Some(HttpResponse::BadRequest().json(error_response("Token cannot be empty.")));
    }
    if password.is_empty() {
        return Some(HttpResponse::BadRequest().json(error_response("Password cannot be empty.")));
    }
    if let Some(confirmation_password) = confirmation_password {
        if confirmation_password.is_empty() {
            return Some(
                HttpResponse::BadRequest()
                    .json(error_response("Confirmation Password cannot be empty.")),
            );
        }
        if confirmation_password != password {
            return Some(
                HttpResponse::BadRequest()
                    .json(error_response("Confirmation Password is incorrect.")),
            );
        }
    }
    None
}

pub fn create_user(username: String, password: String) -> User {
    let mut user = default_user();
    user.username = username;
    user.password = password;
    user
}

pub async fn insert_user(collection: &Collection<Document>, user: User) -> Result<(), String> {
    let user_doc = match to_document(&user) {
        Ok(doc) => doc,
        Err(err) => return Err(format!("Error converting user to BSON: {}", err)),
    };

    match collection.insert_one(user_doc).await {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Error inserting user: {}", err)),
    }
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

pub async fn is_user_exists(
    collection: &Collection<Document>,
    username: &str,
) -> Result<bool, Error> {
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

fn create_score_entry(score_update: &ScoreUpdateRequest) -> Score {
    Score {
        wpm: score_update.score.wpm,
        raw_wpm: score_update.score.raw_wpm,
        accuracy: score_update.score.accuracy,
        date: score_update.score.date,
    }
}

pub fn update_user_high_scores(user: &mut User, score_update: ScoreUpdateRequest) {
    let new_entry = create_score_entry(&score_update);

    // Extract relevant details from score_update
    let timer_duration_str = score_update.duration;
    let language = score_update.language;
    let difficulty = score_update.difficulty;

    user.high_scores = Some(update_high_scores(
        user.high_scores.take(),
        language,
        difficulty,
        timer_duration_str,
        new_entry,
    ));
    increment_completed_tests(&mut user.completed_tests);
}

fn update_high_scores(
    existing_high_scores: Option<HighScores>,
    language: String,
    difficulty: String,
    timer_duration_str: String,
    new_entry: Score,
) -> HighScores {
    let mut high_scores = existing_high_scores.unwrap_or_else(|| HighScores {
        languages: HashMap::new(),
    });

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

    // Update score if new entry is better
    if new_entry.wpm > existing_score.wpm {
        *existing_score = new_entry;
    }

    high_scores
}

fn increment_completed_tests(completed_tests: &mut Option<u32>) {
    if let Some(tests) = completed_tests {
        *tests += 1;
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
