use crate::controllers::user_controller::{add_user, get_user};
use actix_web::web;

pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/add_user", web::post().to(add_user))
        .route("/get_user/{username}", web::get().to(get_user));
}
