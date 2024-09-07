use crate::constants::{COLL_NAME, DB_NAME};
use crate::models::api_response::{error_response, success_response}; // Import the helper functions
use crate::models::user::{default_user, User};
use actix_web::{web, HttpResponse};
use mongodb::{bson::doc, Client, Collection};

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
