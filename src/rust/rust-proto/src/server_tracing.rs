pub fn server_trace_fn<T>(request: &http::Request<T>) -> tracing::Span {
    let headers = request.headers();
    tracing::info_span!(
        "server_trace_fn",
        request_id = ?headers.get("x-request-id"),
        trace_id = ?headers.get("x-trace-id"),
        method = ?request.method(),
        extensions = ?request.extensions(),
    )
}
