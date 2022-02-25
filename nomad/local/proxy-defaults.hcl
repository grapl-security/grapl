#Kind = "proxy-defaults"
#Name = "global"

Config {

  # define the default protocol for tracing to use.
  protocol = "grpc"

  #### Metrics config

  # example configs for statsd or dogstats if we wanted to use those instead
  # (dog)statsd listener on either UDP or Unix socket.
  # envoy_statsd_url = "udp://127.0.0.1:9125"
  # envoy_dogstatsd_url = "udp://127.0.0.1:9125"

  # Prometheus config, which we will be using
  # IP:port to expose the /metrics endpoint on for scraping.
  prometheus_bind_addr = "0.0.0.0:9102"

  # The flush interval in seconds.
  envoy_stats_flush_interval = "10s"
  ### Tracing config
  # This example uses Zipkin, but Consul supports Jaegar, OpenTracing, Honeycomb and others.
  # As of Consul 1.10 Consul uses xDS v3 syntax.

  # This is the tracing config and where we configure which tracing format.
  envoy_tracing_json               = <<EOF
{
  "http": {
    "name": "envoy.tracers.zipkin",
    "typedConfig": {
      "@type": "type.googleapis.com/envoy.config.trace.v3.ZipkinConfig",
      "collector_cluster": "zipkin",
      "collector_endpoint_version": "HTTP_JSON",
      "collector_endpoint": "/api/v1/spans",
      "shared_span_context": false
    }
  }
}
EOF

  # This is used to configure the endpoint(s) for the tracing backend
  envoy_extra_static_clusters_json = <<EOF2
{
  "name": "zipkin",
  "type": "STRICT_DNS",
  "connect_timeout": "5s",
  "load_assignment": {
    "cluster_name": "zipkin",
    "endpoints": [
      {
        "lb_endpoints": [
          {
            "endpoint": {
              "address": {
                "socket_address": {
                  "address": "100.115.92.202",
                  "port_value": 9411
                }
              }
            }
          }
        ]
      }
    ]
  }
}
EOF2
}