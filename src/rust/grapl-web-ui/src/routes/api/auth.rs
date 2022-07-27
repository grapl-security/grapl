mod check_login;
mod sign_in_with_google;
mod sign_in_with_password;

use actix_web::web;
use check_login::check_login;
use sign_in_with_google::sign_in_with_google;
use sign_in_with_password::sign_in_with_password;

pub(super) fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/checkLogin", web::post().to(check_login))
        .route("/sign_in_with_google", web::post().to(sign_in_with_google))
        .route(
            "/sign_in_with_password",
            web::post().to(sign_in_with_password),
        );
}
