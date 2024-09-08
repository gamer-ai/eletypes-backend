use crate::controllers::leaderboard_controller::{get_leaderboard_stats, update_user_score};
use actix_web::web;

pub fn configure_leaderboard_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/get_leaderboard_stats",
        web::get().to(get_leaderboard_stats),
    )
    .route(
        "/update_user_score/{username}",
        web::post().to(update_user_score),
    );
}
