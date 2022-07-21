use std::ops::Deref;

use actix_web::{
    post,
    web,
    HttpRequest,
    HttpResponse,
    Result,
};
use actix_web_opentelemetry::ClientExt;

pub(super) fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(handler);
}

#[post("/{tail:.*}")]
pub(crate) async fn handler(
    req: HttpRequest,
    payload: web::Payload,
    backend_endpoint: web::Data<crate::GraphQlEndpointUrl>,
    path: web::Path<String>,
    client: web::Data<awc::Client>,
    _user: crate::authn::AuthenticatedUser,
) -> Result<HttpResponse> {
    // Strip "api/graphQlEndpoint" from the request URL before forwarding.
    let backend_endpoint = backend_endpoint.get_ref().deref().clone();
    let backend_path = path.into_inner();
    let mut backend_url = backend_endpoint;
    backend_url.set_path(backend_path.as_str());
    backend_url.set_query(req.uri().query());

    tracing::debug!(
        message = "Forwarding request to graphQL backend",
        %backend_url,
    );

    fwd_request_to_backend_service(req, payload, backend_url, client.get_ref().clone()).await
}

// This is for routing requests to HTTP backend services. This implementation
// is designed to match the actix_web http-proxy example at:
// https://github.com/actix/examples/blob/master/basics/http-proxy/src/main.rs
#[tracing::instrument(skip(client, req, payload))]
pub(self) async fn fwd_request_to_backend_service(
    req: HttpRequest,
    payload: web::Payload,
    backend_url: url::Url,
    client: awc::Client,
) -> Result<HttpResponse> {
    //TODO(inickles): handle X-Forwarded-For/Forwarded headers
    let forwarded_req = client
        .request_from(backend_url.as_str(), req.head())
        .no_decompress();

    let mut res = forwarded_req
        .trace_request()
        .send_stream(payload)
        .await
        .map_err(|error| {
            tracing::error!(%error);

            actix_web::error::ErrorInternalServerError(error)
        })?;

    tracing::debug!(
        message = "Received response from backend service",
        %backend_url,
        response = ?res
    );

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.append_header((header_name.clone(), header_value.clone()));
    }

    Ok(client_resp.body(res.body().await?))
}
