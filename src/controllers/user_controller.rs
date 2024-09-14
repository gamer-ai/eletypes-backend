use crate::constants::{COLL_NAME, DB_NAME};
use crate::models::user::User;
use crate::services::user_service::{
    create_user, fetch_user_and_handle_response, insert_user, is_user_exists, save_user_scores,
    update_user_high_scores, verify_recaptcha,
};
use crate::structs::api_response::{error_response, success_response};
use crate::structs::leaderboard::ScoreUpdateRequest;
use crate::structs::login::LoginRequest;

use actix_web::{web, HttpResponse};
use mongodb::{
    bson::{doc, Document},
    Client, Collection,
};

fn get_collection(client: &Client) -> Collection<User> {
    client.database(DB_NAME).collection(COLL_NAME)
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

pub async fn sign_up(client: web::Data<Client>, form: web::Json<LoginRequest>) -> HttpResponse {
    let collection = get_collection(&client);
    let request_data = form.into_inner();

    let username = request_data.username.trim();
    let token = request_data.token.trim();
    let password = request_data.password.trim();
    let confirmation_password = request_data.confirmation_password.trim();

    if username.is_empty() {
        return HttpResponse::BadRequest().json(error_response("Username cannot be empty."));
    }
    if token.is_empty() {
        return HttpResponse::BadRequest().json(error_response("Token cannot be empty."));
    }
    if password.is_empty() {
        return HttpResponse::BadRequest().json(error_response("Password cannot be empty."));
    }
    if confirmation_password.is_empty() {
        return HttpResponse::BadRequest()
            .json(error_response("Confirmation Password cannot be empty."));
    }
    if confirmation_password != password {
        return HttpResponse::BadRequest()
            .json(error_response("Confirmation Password is incorrect."));
    }

    match verify_recaptcha(token).await {
        Ok(response) => {
            if response.success {
                println!("reCAPTCHA verification successful");
            } else {
                return HttpResponse::BadRequest().json(error_response(&format!(
                    "reCAPTCHA verification failed: {:?}",
                    response.error_codes
                )));
            }
        }
        Err(err) => {
            eprintln!("Error verifying reCAPTCHA: {:?}", err);
            return HttpResponse::InternalServerError()
                .json(error_response("Error verifying reCAPTCHA."));
        }
    }

    match is_user_exists(&collection, username).await {
        Ok(true) => HttpResponse::BadRequest().json(error_response("Username already taken.")),
        Ok(false) => {
            let user = create_user(username.to_string(), password.to_string());

            match insert_user(&collection, user).await {
                Ok(_) => HttpResponse::Ok().json(success_response("User successfully registered.")),
                Err(err) => HttpResponse::InternalServerError().json(error_response(&err)),
            }
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(error_response("Error checking username availability.")),
    }
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
