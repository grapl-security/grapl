pub mod create;
mod error;
pub mod get_metadata;

use actix_web::web;
pub use error::PluginError;

pub(super) fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/create", web::post().to(create::create));
    cfg.route("/get_metadata", web::get().to(get_metadata::get_metadata));
}
