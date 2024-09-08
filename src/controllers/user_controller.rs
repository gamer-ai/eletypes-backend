use crate::constants::{COLL_NAME, DB_NAME};
use crate::models::api_response::{error_response, success_response};
use crate::models::leaderboard::ScoreUpdateRequest;
use crate::models::user::{default_user, User};
use crate::services::user_service::{
    fetch_user_and_handle_response, save_user_scores, update_user_high_scores,
};

use actix_web::{web, HttpResponse};
use mongodb::{
    bson::{doc, Document},
    Client, Collection,
};

fn get_collection(client: &Client) -> Collection<User> {
    client.database(DB_NAME).collection(COLL_NAME)
}

fn handle_error<T>(result: Result<T, mongodb::error::Error>, error_msg: &str) -> HttpResponse {
    match result {
        Ok(_) => HttpResponse::Ok().json(success_response("User added successfully.")),
        Err(err) => HttpResponse::InternalServerError()
            .json(error_response(&format!("{}: {}", error_msg, err))),
    }
}

pub async fn update_user_scores(
    client: web::Data<Client>,
    username: web::Path<String>,
    form: web::Json<ScoreUpdateRequest>,
) -> HttpResponse {
    let collection = client.database(DB_NAME).collection::<Document>(COLL_NAME);
    let username_str = username.into_inner();
    let score_update = form.into_inner();

    let mut user = match fetch_user_and_handle_response(&collection, &username_str).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    update_user_high_scores(&mut user, score_update);

    save_user_scores(&collection, &username_str, &user).await
}

pub async fn add_user(client: web::Data<Client>, form: web::Json<User>) -> HttpResponse {
    let collection = get_collection(&client);
    let mut user = form.into_inner();

    if user.username.trim().is_empty() {
        return HttpResponse::BadRequest().json(error_response("Username cannot be empty."));
    }
    if user.password.trim().is_empty() {
        return HttpResponse::BadRequest().json(error_response("Password cannot be empty."));
    }

    if user.high_scores.is_none() {
        user.high_scores = default_user().high_scores;
    }

    let result = collection.insert_one(user).await;

    handle_error(result.map(|_| ()), "Error adding user")
}

pub async fn get_user(client: web::Data<Client>, username: web::Path<String>) -> HttpResponse {
    let collection = get_collection(&client);
    let username = username.into_inner();
    match collection.find_one(doc! { "username": &username }).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().json(error_response(&format!(
            "No user found with username '{}'",
            username
        ))),
        Err(err) => HttpResponse::InternalServerError()
            .json(error_response(&format!("Error retrieving user: {}", err))),
    }
}
