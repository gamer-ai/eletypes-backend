use actix_cors::Cors;
use actix_web::http;

pub fn configure_cors() -> Cors {
    Cors::default()
        .allowed_origin("http://localhost:5173")
        .allowed_origin("http://localhost:3000")
        .allowed_origin("https://eletypes.com")
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE])
        .supports_credentials()
        .max_age(3600) // Cache preflight responses for 1 hour
}
