use crate::controllers::user_controller::{get_user, sign_up, update_user_scores};
use actix_web::web;

pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/sign_up", web::post().to(sign_up))
        .route("/get_user/{username}", web::get().to(get_user))
        .route(
            "/update_user_scores/{username}",
            web::post().to(update_user_scores),
        );
}
