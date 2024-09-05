use crate::constants::{COLL_NAME, DB_NAME};
use crate::models::score_model::ScoreEntry;
use crate::repositories::user_repository::update_user_high_scores;
use crate::repositories::user_repository::{
    fetch_user_and_handle_response, update_user_and_handle_respond,
};
use actix_web::{web, HttpResponse};
use mongodb::bson;
use mongodb::{bson::doc, Client};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ScoreUpdateRequest {
    pub score: ScoreEntry,
    pub timer_duration: u32,
    pub test_completed: u32,
}

pub async fn update_user_score(
    client: web::Data<Client>,
    username: web::Path<String>,
    form: web::Json<ScoreUpdateRequest>,
) -> HttpResponse {
    let collection = client
        .database(DB_NAME)
        .collection::<bson::Document>(COLL_NAME);
    let username_str = username.into_inner();
    let score_update = form.into_inner();

    // Fetch user from the database
    let mut user = match fetch_user_and_handle_response(&collection, &username_str).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    // Update user data
    update_user_high_scores(&mut user, score_update);

    // Update the user in the database
    update_user_and_handle_respond(&collection, &username_str, &user).await
}

pub async fn get_leaderboard_stats(
    _client: web::Data<Client>,
    _form: web::Json<ScoreUpdateRequest>,
) {
}
