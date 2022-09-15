from infra import config

import pulumi


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

    # Default is 512. We were getting errors that our Thrift messages were too big.
    otel_bsp_max_export_batch_size = 32

    return f"""
        OTEL_EXPORTER_JAEGER_AGENT_HOST = {otel_host}
        OTEL_EXPORTER_JAEGER_AGENT_PORT = {otel_port}
        OTEL_EXPORTER_ZIPKIN_ENDPOINT   = {zipkin_endpoint}
        OTEL_BSP_MAX_EXPORT_BATCH_SIZE = {otel_bsp_max_export_batch_size}
    """


# lightstep_token should be pulumi.Output[str], but the additional type causes pulumi.Output.all to blow up during
# typechecking
def otel_config(
    lightstep_token: pulumi.Output,
    lightstep_endpoint: str = "ingest.lightstep.com:443",
    lightstep_is_endpoint_secure: str = "true",
) -> pulumi.Output[str]:
    return pulumi.Output.all(
        lightstep_endpoint=lightstep_endpoint,
        lightstep_token=lightstep_token,
        lightstep_is_endpoint_secure=lightstep_is_endpoint_secure,
    ).apply(
        lambda args: f"""
receivers:
  zipkin:
  jaeger:
    protocols:
      thrift_compact:
        endpoint: "0.0.0.0:6831"
  otlp:
    protocols:
      grpc:
      http:
        endpoint: "0.0.0.0:4318"
  prometheus:
    config:
      scrape_configs:
        - job_name: 'nomad-server'
          scrape_interval: 20s
          scrape_timeout: 10s
          metrics_path: '/v1/metrics'
          params:
            format: ['prometheus']
          static_configs:
            # This should be pointing to the Nomad network IP. Locally that would be the eth0 private IP. in AWS that's the Nomad ALB.
            - targets: ['localhost:4646']
processors:
  batch:
    timeout: 10s
  memory_limiter:
    # 75% of maximum memory up to 4G
    limit_mib: 1536
    # 25% of limit up to 2G
    spike_limit_mib: 512
    check_interval: 5s
exporters:
  logging:
    logLevel: debug
  otlp/ls:
    endpoint: {args['lightstep_endpoint']}
    tls:
      insecure: {args['lightstep_is_endpoint_secure']}
    headers:
      "lightstep-access-token": {args['lightstep_token']}
service:
  telemetry:
    logs:
      level: "debug"
  pipelines:
    metrics:
      receivers: [prometheus]
      exporters: [otlp/ls]
    traces:
      receivers: [jaeger, otlp, zipkin]
      exporters: [logging, otlp/ls]
"""
    )
