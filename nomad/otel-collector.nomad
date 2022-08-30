job "otel-collector-gateway" {

  datacenters = ["dc1"]

  group "svc" {
    count = 1

    network {

      port "otlp-grpc" {
        to = 4317
      }

      port "otlp-http" {
        to = 4318
      }

      port "metrics" {
        to = 8888
      }

      # Receivers
      port "prometheus" {
        to = 9090
      }

      port "zipkin" {
        to = 9411
      }

      port "jaeger-grpc" {
        to = 14250
      }

      port "jaeger-thrift" {
        to = 14268
      }
    }

    service {
      port = "otlp-http"
    }

    service {
      name = "otel-collector-hc"
      port = "prometheus"
      tags = ["prometheus"]
    }

    service {
      name = "otel-collector-zipkin"
      port = "zipkin"
      tags = ["zipkin"]
    }

    service {
      name = "otel-agent-hc"
      port = "metrics"
      tags = ["metrics"]
    }

    task "svc" {
      driver = "docker"

      config {
        image      = "otel/opentelemetry-collector-contrib:0.40.0"
        force_pull = true

        entrypoint = [
          "/otelcontribcol",
          "--config=local/config/otel-collector-config.yaml",
        ]
        ports = [
          "metrics",
          "prometheus",
          "otlp-grpc",
          "otlp-http"
        ]
      }


      resources {
        cpu    = 100
        memory = 512
      }

      template {
        data        = <<EOF
receivers:
  zipkin:
  otlp:
    protocols:
      grpc:
      http:
        endpoint: "0.0.0.0:4318"
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
    # to see spans in logs update logLevel to debug
    logLevel: info
  otlp/ls:
    endpoint: ingest.lightstep.com:443
    headers:
      "lightstep-access-token": "leVpbNbMm++jqMMyA0N6Ko+Acuzw7xWTw3yxenVkVll4fzxY2VwmeQwPOIuTqUK/yhfxJeGsGorY4Qk891ngSipYm/agwwox2aggLY7h"
service:
  pipelines:
    traces:
      receivers: [otlp]
      exporters: [logging, otlp/ls]
EOF
        destination = "local/config/otel-collector-config.yaml"
      }
    }
  }
}