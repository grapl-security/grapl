variable "otel_config" {
  # We're setting a default here because of what appears to be a silent type error during preview
  # Oddly, the reflection runs correctly, but the variable we pass in seems to be unset at some point?
  type        = string
  default     = ""
  description = <<EOF
We inject the whole yaml config for the otel collector.
Long-term, this will likely be done in-line with secrets grabbed dynamically from Vault.
This requires that Nomad and Vault be hooked up first
EOF
}

job "observability" {
  datacenters = ["dc1"]
  type        = "system"

  group "otel-collector" {
    count = 1

    network {

      port "metrics" {
        to = 8888
      }

      # Receivers
      port "otlp-grpc" {
        to = 4317
      }

      port "otlp-http" {
        to = 4318
      }

      port "jaeger-thrift-compact" {
        to     = 6831
        static = 6831
      }

      port "prometheus" {
        to     = 9090
        static = 9090
      }

      port "zipkin" {
        to     = 9411
        static = 9411
      }
    }

    service {
      name = "prometheus"
      port = "prometheus"
      tags = ["prometheus"]
    }

    service {
      port = "otlp-http"
    }

    service {
      name = "otel-collector-zipkin"
      port = "zipkin"
      tags = ["zipkin"]
    }

    service {
      name = "otel-collector-jaeger-thrift-compact"
      port = "jaeger-thrift-compact"
      tags = ["jaeger"]
    }

    service {
      name = "otel-agent-hc"
      port = "metrics"
      tags = ["metrics"]
    }

    task "otel-collector" {
      driver = "docker"

      config {
        image      = "otel/opentelemetry-collector-contrib:0.40.0"
        force_pull = true

        entrypoint = [
          "/otelcontribcol",
          "--config=local/config/otel-collector-config.yaml",
        ]
        ports = [
          "jaeger-thrift-compact",
          "metrics",
          "otlp-grpc",
          "otlp-http",
          "prometheus",
          "zipkin"
        ]
      }


      resources {
        cpu    = 100
        memory = 512
      }

      template {
        data        = var.otel_config
        destination = "local/config/otel-collector-config.yaml"
      }
    }
  }
}