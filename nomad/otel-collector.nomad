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
      name = "otel-agent-hc"
      port = "metrics"
      tags = ["metrics"]
    }

    task "svc" {
      driver = "docker"

      config {
        image = "otel/opentelemetry-collector-contrib:0.40.0"
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
        data   = <<EOF
receivers:
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
    logLevel: debug
service:
  pipelines:
    traces:
      receivers: [otlp]
      exporters: [logging]
EOF
        destination = "local/config/otel-collector-config.yaml"
      }
    }
  }
}