use crate::controllers::leaderboard_controller::get_leaderboard_stats;
use actix_web::web;

pub fn configure_leaderboard_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/get_leaderboard_stats",
        web::get().to(get_leaderboard_stats),
    );
}
