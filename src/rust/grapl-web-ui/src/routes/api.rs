mod auth;
mod graphql;

pub(super) fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/auth").configure(auth::config));
    cfg.service(actix_web::web::scope("/graphQlEndpoint").configure(graphql::config));
}
