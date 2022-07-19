from infra import config


def observability_env_vars_for_local() -> str:
    # We currently use both the zipkin v2 endpoint for consul, python and typescript instrumentation and the jaeger udp
    # agent endpoint for rust instrumentation. These will be consolidated in the future

    if not config.LOCAL_GRAPL:
        return "DUMMY_VAR_FOR_PROD = TRUE"

    # These use the weird Mustache {{}} tags because this interpolation eventually
    # gets passed in to a template{} stanza.
    otel_host = '{{ env "attr.unique.network.ip-address" }}'
    otel_port = 6831  # compare with nomad/local/observability.nomad
    zipkin_endpoint = (
        'http://{{ env "attr.unique.network.ip-address" }}:9411/api/v2/spans'
    )

    return f"""
        OTEL_EXPORTER_JAEGER_AGENT_HOST = {otel_host}
        OTEL_EXPORTER_JAEGER_AGENT_PORT = {otel_port}
        OTEL_EXPORTER_ZIPKIN_ENDPOINT   = {zipkin_endpoint}
    """
