mod error;
pub mod publish;

use actix_web::web;
pub use error::IngressError;

pub(super) fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/publish/{event_source_id}",
        web::post().to(publish::publish),
    );
}
