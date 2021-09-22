use actix_files as fs;
use actix_web::{
    guard,
    web,
    Responder,
};

pub(super) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/grapl_logo.png")
            .route(web::get().to(get))
            .guard(guard::Get()),
    );
}

async fn get() -> impl Responder {
    fs::NamedFile::open("frontend/grapl_logo.png")
}
