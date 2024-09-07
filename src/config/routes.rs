use crate::routes::score_routes::{get_leaderboard_stats, update_user_score};
use crate::routes::user_routes::{add_user, get_user};
use actix_web::web;

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
