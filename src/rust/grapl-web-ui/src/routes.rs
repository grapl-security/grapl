mod api;
mod grapl_logo;
pub mod index;
mod r#static;

use actix_web::{
    guard,
    web,
};

pub(super) fn config(cfg: &mut actix_web::web::ServiceConfig) {
    grapl_logo::config(cfg);
    index::config(cfg);
    r#static::config(cfg);
    cfg.service(
        web::scope("/api")
            .configure(api::config)
            .guard(guard::Post())
            // .guard(guard::Header("X-Requested-With", "XMLHttpRequest"))
            .guard(guard::Header("content-type", "application/json")),
    );
}
