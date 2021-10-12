pub mod graphql;
pub mod model_plugin_deployer;

use actix_web::{
    client,
    web,
    HttpRequest,
    HttpResponse,
    Result,
};

// This is for routing requests to HTTP backend services. This implementation
// is designed to match the actix_web http-proxy example at:
// https://github.com/actix/examples/blob/master/basics/http-proxy/src/main.rs
//
// TODO(inickles): in the future we should probably drop the first directory of the URL path, so
// the scope paths here don't need to match that of the backend.
#[tracing::instrument(skip(client, req))]
pub(self) async fn fwd_request_to_backend_service(
    req: HttpRequest,
    body: web::Bytes,
    backend_endpoint: url::Url,
    client: client::Client,
) -> Result<HttpResponse> {
    let mut new_url = backend_endpoint;
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());

    tracing::debug!(
        message = "Forwarding request to backend service",
        backend_url = %new_url,
    );

    //TODO(inickles): handle X-Forwarded-For/Forwarded headers
    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();

    let mut res = forwarded_req
        .send_body(body)
        .await
        .map_err(actix_web::Error::from)?;

    tracing::debug!(
        message = "Received response from backend service",
        backend_url = %new_url,
        response = ?res
    );

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.header(header_name.clone(), header_value.clone());
    }

    Ok(client_resp.body(res.body().await?))
}
