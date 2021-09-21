use actix_files as fs;
use actix_web::{
    web,
    Responder,
};

pub(super) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(index)));
}

pub async fn index() -> impl Responder {
    fs::NamedFile::open("frontend/index.html")
}
