use crate::routes::score_routes::{get_leaderboard_stats, update_user_score};
use crate::routes::user_routes::{add_user, get_user};
use actix_web::web;
use dotenv::dotenv;
use mongodb::Client;

pub async fn connect_to_mongodb() -> Client {
    dotenv().ok();

    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    match Client::with_uri_str(uri).await {
        Ok(client) => client,
        Err(e) => {
            panic!("Failed to connect to MongoDB: {:?}", e);
        }
    }
}

pub fn get_server_address() -> String {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    format!("localhost:{}", port)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/get_user/{username}", web::get().to(get_user))
        .route("/add_user", web::post().to(add_user))
        .route(
            "/update_user_score/{username}",
            web::post().to(update_user_score),
        )
        .route(
            "/get_leaderboard_stats",
            web::get().to(get_leaderboard_stats),
        );
}
