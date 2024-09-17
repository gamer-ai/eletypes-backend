use crate::controllers::user_controller::{
    check_auth, get_user_detail, login, logout, sign_up, update_user_scores,
};
use actix_web::web;

pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/sign_up", web::post().to(sign_up))
        .route("/login", web::post().to(login))
        .route("/check_auth", web::get().to(check_auth))
        .route(
            "/get_user_detail/{username}",
            web::get().to(get_user_detail),
        )
        .route(
            "/update_user_scores/{username}",
            web::post().to(update_user_scores),
        )
        .service(web::resource("/logout").route(web::delete().to(logout)));
}
