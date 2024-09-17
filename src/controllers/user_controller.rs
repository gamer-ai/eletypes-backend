use crate::services::user_service::{
    authenticate_user, create_http_only_cookie, fetch_user_and_handle_response, generate_jwt,
    process_user_registration, save_user_scores, update_user_high_scores, validate_credentials,
    verify_recaptcha, verify_recaptcha_and_check,
};
use crate::structs::api_response::{error_response, success_response, success_response_with_data};
use crate::structs::claims::Claims;
use crate::structs::leaderboard::ScoreUpdateRequest;
use crate::structs::login::LoginRequest;
use crate::structs::sign_up::SignUpRequest;
use crate::utils::helpers::get_collection;
use actix_web::cookie::time::Duration;
use actix_web::{cookie::Cookie, web, Error, HttpRequest, HttpResponse};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use mongodb::{bson::doc, Client};
use std::env;

pub async fn logout(_req: HttpRequest) -> Result<HttpResponse, Error> {
    let cookie = Cookie::build("user_jwt_token", "")
        .http_only(true)
        .secure(true)
        .path("/")
        .max_age(Duration::new(0, 0))
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(cookie)
        .json(success_response("Logout successful.")))
}

pub async fn check_auth(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Attempt to extract the "Authorization" cookie
    let cookie = match req.cookie("user_jwt_token") {
        Some(cookie) => cookie,
        None => return Ok(HttpResponse::Unauthorized().finish()), // Guard clause: No cookie
    };

    let token = cookie.value();

    // Fetch JWT secret key from environment variable
    let secret_key = match env::var("JWT_SECRET") {
        Ok(key) => key,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()), // Guard clause: Missing secret key
    };

    // Attempt to decode the JWT
    let decoding_key = DecodingKey::from_secret(secret_key.as_ref());
    let validation = Validation::new(Algorithm::HS256);

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(_decoded_token) => Ok(HttpResponse::Ok().finish()), // Token is valid
        Err(err) => {
            eprintln!("JWT decode error: {:?}", err); // Log the decoding error
            Ok(HttpResponse::Unauthorized().finish()) // Token is invalid
        }
    }
}

pub async fn update_user_scores(
    client: web::Data<Client>,
    username: web::Path<String>,
    req: web::Json<ScoreUpdateRequest>,
) -> HttpResponse {
    let collection = get_collection(&client);
    let username_str = username.into_inner();
    let score_update = req.into_inner();

    match fetch_user_and_handle_response(&collection, &username_str).await {
        Ok(mut user) => {
            update_user_high_scores(&mut user, score_update);
            save_user_scores(&collection, &username_str, &user).await
        }
        Err(response) => response,
    }
}

pub async fn sign_up(client: web::Data<Client>, req: web::Json<SignUpRequest>) -> HttpResponse {
    let collection = get_collection(&client);
    let sign_up_request = req.into_inner();

    let username = sign_up_request.username.trim();
    let recaptcha_token = sign_up_request.token.trim();
    let password = sign_up_request.password.trim();
    let confirmation_password = sign_up_request.confirmation_password.trim();

    if let Some(response) = validate_credentials(
        username,
        recaptcha_token,
        password,
        Some(confirmation_password),
    ) {
        return response;
    }

    let recaptcha_response = match verify_recaptcha(recaptcha_token).await {
        Ok(response) => response,
        Err(err) => {
            eprintln!("Error verifying reCAPTCHA: {:?}", err);
            return HttpResponse::InternalServerError()
                .json(error_response("Error verifying reCAPTCHA."));
        }
    };

    if !recaptcha_response.success {
        return HttpResponse::BadRequest().json(error_response(&format!(
            "reCAPTCHA verification failed: {:?}",
            recaptcha_response.error_codes
        )));
    }

    process_user_registration(&collection, username, password).await
}

pub async fn login(client: web::Data<Client>, req: web::Json<LoginRequest>) -> HttpResponse {
    let collection = get_collection(&client);
    let login_request = req.into_inner();

    let username = login_request.username.trim();
    let recaptcha_token = login_request.token.trim();
    let password = login_request.password.trim();

    // Validate credentials and return early if there is an error
    if let Some(response) = validate_credentials(username, recaptcha_token, password, None) {
        return response;
    }

    // Verify reCAPTCHA token and return early if there is an error
    if let Err(response) = verify_recaptcha_and_check(recaptcha_token).await {
        return response;
    }

    // Authenticate user
    let is_authenticated = match authenticate_user(&collection, username, password).await {
        Ok(is_authenticated) => is_authenticated,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(error_response("Error authenticating user."))
        }
    };

    // Handle authentication result
    if !is_authenticated {
        return HttpResponse::Unauthorized().json(error_response("Invalid username or password."));
    }

    // Generate JWT token
    let jwt_token = match generate_jwt(username) {
        Ok(token) => token,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(error_response("Error generating JWT token."))
        }
    };

    // Create and return HTTP response with JWT token in a cookie
    let cookie = create_http_only_cookie(jwt_token);
    HttpResponse::Ok()
        .cookie(cookie)
        .json(success_response_with_data("Login successful.", username))
}

pub async fn get_user_detail(
    client: web::Data<Client>,
    username: web::Path<String>,
) -> HttpResponse {
    let collection = get_collection(&client);
    let username = username.into_inner();

    let filter = doc! { "username": &username };

    match collection.find_one(filter).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().json(error_response(&format!(
            "No user found with username '{}'",
            username
        ))),
        Err(err) => HttpResponse::InternalServerError()
            .json(error_response(&format!("Error retrieving user: {}", err))),
    }
}
