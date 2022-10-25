import pulumi


def get_observability_env_vars() -> str:
    # We currently use both the zipkin v2 endpoint for consul, python and typescript instrumentation and the jaeger udp
    # agent endpoint for rust instrumentation. These will be consolidated in the future

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
    consul_agent_endpoint: str,
    consul_l7_metric_endpoint: str,
    nomad_agent_endpoint: str,
    lightstep_endpoint: str,
    # This is optional because pulumi.config.get_bool returns Optional[bool]
    lightstep_is_endpoint_insecure: bool | None,
    trace_sampling_percentage: float | None,
) -> pulumi.Output[str]:
    return pulumi.Output.all(
        lightstep_endpoint=lightstep_endpoint,
        lightstep_token=lightstep_token,
        lightstep_is_endpoint_insecure=lightstep_is_endpoint_insecure,
        consul_agent_endpoint=consul_agent_endpoint,
        consul_l7_metric_endpoint=consul_l7_metric_endpoint,
        nomad_agent_endpoint=nomad_agent_endpoint,
        trace_sampling_percentage=trace_sampling_percentage,
    ).apply(
        lambda args: rf"""
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
        # - job_name: 'consul-agent'
        #   scrape_interval: 20s
        #   scrape_timeout: 10s
        #   metrics_path: '/v1/agent/metrics'
        #   params:
        #     format: ['prometheus']
        #   static_configs:
        #     - targets: [{args["consul_agent_endpoint"]}]
        - job_name: 'consul-envoy'
          scrape_interval: 20s
          scrape_timeout: 10s
          consul_sd_configs:
            - server: {args["consul_agent_endpoint"]}
          relabel_configs:
            # Don't scrape the extra -sidecar-proxy services that Consul Connect
            # sets up, otherwise we'll have duplicates.
            - source_labels: [__meta_consul_service]
              action: drop
              regex: (.+)-sidecar-proxy
            # Only scrape services that have a metrics_port meta field.
            # This is optional, you can use other criteria to decide what
            # to scrape.
            - source_labels: [__meta_consul_service_metadata_metrics_port_envoy]
              action: keep
              regex: (.+)
            # Replace the port in the address with the one from the metrics_port
            # meta field.
            - source_labels: [__address__, __meta_consul_service_metadata_metrics_port_envoy]
              regex: ([^:]+)(?::\d+)?;(\d+)
              replacement: ${1}:${2}
              target_label: __address__
          static_configs:
            - targets: [{args["consul_l7_metric_endpoint"]}]
        # - job_name: 'nomad-agent'
        #   scrape_interval: 20s
        #   scrape_timeout: 10s
        #   metrics_path: '/v1/metrics'
        #   params:
        #     format: ['prometheus']
        #   static_configs:
        #     - targets: [{args["nomad_agent_endpoint"]}]
processors:
  batch:
    timeout: 10s
  memory_limiter:
    # 75% of maximum memory up to 4G
    limit_mib: 1536
    # 25% of limit up to 2G
    spike_limit_mib: 512
    check_interval: 5s
  # This sets up head-based sampling as the simplest way to add sampling. Ideally, we'd use tail-based sampling which 
  # allows for rule-based sampling. Unfortunately tail-based sampling doesn't work well with multiple collector 
  # instances. For it to work, you need agents doing load-balancer exporting to a dedicated processing cluster/service,
  # which is overkill for our current scale.
  # TODO create a dedicated processing cluster and switch over to tail-based sampling.
  probabilistic_sampler:
    sampling_percentage: {args["trace_sampling_percentage"]}
exporters:
  logging:
    logLevel: debug
  otlp/ls:
    endpoint: {args['lightstep_endpoint']}
    tls:
      insecure: {args['lightstep_is_endpoint_insecure']}
    headers: 
      "lightstep-access-token": "{args['lightstep_token']}"
service:
  telemetry:
    logs:
      level: "debug"
  pipelines:
    metrics:
      receivers: [prometheus]
      processors: [batch]
      # To enable debug logging, add logging to exporters
      exporters: [logging, otlp/ls]
    traces:
      receivers: [jaeger, otlp, zipkin]
      processors: [batch, memory_limiter, probabilistic_sampler]
      # To enable debug logging, add logging to exporters
      exporters: [otlp/ls]
"""
    )
