mod auth;
mod health;

use actix_web::web;

pub(super) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").configure(auth::config));
    cfg.route("/health", web::get().to(health::health));
}
