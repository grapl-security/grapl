# This contains the observability jobs.
# First up is Jaegar so that we can collect collect and see traces during local development.
job "observability" {
  datacenters = ["dc1"]
  type        = "service"

  # Jaeger
  group "jaeger" {
    network {
      mode = "host"
      # We currently only want a web front-end and a zipkin endpoint exposed. However, Jaegar supports thrift and grpc
      # on other ports. We can expose them if/when we have a use for them..
      port "http-frontend" {
        to     = 16686
        static = 16686
      }

      port "grpc" {
        to = 16685
        static = 16685
      }

      port "jaegar-thrift" {
        to = 14268
        static = 14268
      }

      # This supports zipkin compatible traces
      port "zipkin" {
        to = 9411
        static = 9411
      }
    }

    service {
      name = "jaeger-frontend"
      port = "http-frontend"
      tags = ["http"]

      check {
        type     = "http"
        port     = "http-frontend"
        path     = "/"
        interval = "5s"
        timeout  = "2s"
      }
    }

    # Service for accepting zipkin format traces
    service {
      name = "jaeger-zipkin-trace-endpoint"
      port = "zipkin"
      tags = ["zipkin"]
    }

    service {
      name = "jaeger-thrift"
      port = "jaegar-thrift"
      tags = ["thrift"]
    }

    service {
      name = "grpc"
      port = "grpc"
      tags = ["grpc"]
    }

    task "jaeger-all-in-one" {
      driver = "docker"

      config {
        image = "jaegertracing/all-in-one:latest"
        ports = ["http-frontend", "zipkin", "grpc", "jaegar-thrift"]
        network_mode = "host"
      }

      env {
        COLLECTOR_ZIPKIN_HOST_PORT=9411
      }

      resources {
        cpu    = 200
        memory = 100
      }
    }
  }
}