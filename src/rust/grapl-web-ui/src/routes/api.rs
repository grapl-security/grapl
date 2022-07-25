mod auth;
mod graphql;
mod health;

use actix_web::web;

pub(super) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").configure(auth::config));
    cfg.service(web::scope("/graphQlEndpoint").configure(graphql::config));
    cfg.route("/health", web::get().to(health::health));
}
