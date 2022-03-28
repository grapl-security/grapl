# This sets up Jaeger
job "jaeger" {
  datacenters = ["dc1"]
  type = "service"

  group "jaeger" {
    network {
      mode = "host"
      # We currently expose the web frontend and then several endpoints that accept traces. There are additional
      # endpoints that support traces whose ports we can open up as necessary
      port "http-frontend" {
        to     = 16686
        static = 16686
      }

      port "grpc" {
        to     = 16685
        static = 16685
      }

      port "jaeger-thrift" {
        to     = 14268
        static = 14268
      }

      # This supports zipkin compatible traces
      port "zipkin" {
        to     = 9411
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
      name = "jaeger-zipkin"
      port = "zipkin"
      tags = ["zipkin"]
    }

    service {
      name = "jaeger-thrift"
      port = "jaeger-thrift"
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
        image        = "jaegertracing/all-in-one:latest"
        ports        = ["http-frontend", "zipkin", "grpc", "jaeger-thrift"]
        network_mode = "host"
      }

      env {
        COLLECTOR_ZIPKIN_HOST_PORT = 9411
      }

      resources {
        cpu    = 200
        memory = 100
      }
    }
  }
}