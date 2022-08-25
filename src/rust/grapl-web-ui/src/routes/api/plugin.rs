pub mod create;
mod get_metadata;

use actix_web::web;

pub(super) fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/create", web::post().to(create::create));
    cfg.route("/get_metadata", web::get().to(get_metadata::get_metadata));
}
