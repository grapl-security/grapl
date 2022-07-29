#[tracing::instrument]
pub(super) async fn health() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().finish()
}
