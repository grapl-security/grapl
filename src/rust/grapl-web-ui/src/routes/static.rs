use actix_files as fs;
use actix_web::web;

pub(super) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        fs::Files::new("/static", "frontend/static")
            .use_last_modified(true)
            .use_etag(true),
    );
}
