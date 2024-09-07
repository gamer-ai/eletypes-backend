use actix_cors::Cors;
pub fn configure_cors() -> Cors {
    Cors::default()
        .allowed_origin("http://localhost:3000")
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE])
        .max_age(3600) // Cache preflight responses for 1 hour
}
