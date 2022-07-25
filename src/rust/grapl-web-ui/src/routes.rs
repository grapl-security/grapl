mod api;
mod grapl_logo;
pub mod index;
mod r#static;

use actix_web::web;

pub(super) fn config(cfg: &mut actix_web::web::ServiceConfig) {
    grapl_logo::config(cfg);
    index::config(cfg);
    r#static::config(cfg);
    cfg.service(web::scope("/api").configure(api::config));
}
