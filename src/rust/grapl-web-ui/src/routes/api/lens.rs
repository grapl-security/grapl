mod subscribe;

use actix_web::web;
use subscribe::subscribe;

pub(super) fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/subscribe", web::post().to(subscribe));
}
