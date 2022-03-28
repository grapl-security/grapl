// agent in system job
// goes to a jaeger all in one with memory configured


job "jaeger-agent" {
  datacenters = ["dc1"]
  type        = "system"

  # Jaeger
  group "jaeger-agent" {
    network {
      mode = "host"

      // Rust services use the jaeger agent udp thrift ports
      port "agent-thrift-compact" {
        to     = 6831
        static = 6831
      }

      port "agent-thrift-binary" {
        to     = 6832
        static = 6832
      }

    }


    service {
      name = "jaeger-agent-thrift-compact"
      port = "agent-thrift-compact"
      tags = ["thrift"]
    }

    service {
      name = "jaeger-agent-thrift-binary"
      port = "agent-thrift-compact"
      tags = ["thrift"]
    }


    task "jaeger-agent" {
      driver = "docker"

      config {
        image        = "jaegertracing/all-in-one:latest"
        ports        = ["http-frontend", "zipkin", "grpc", "jaeger-thrift"]
        network_mode = "host"
      }

      env {
        # define the collector url
#        COLLECTOR_ZIPKIN_HOST_PORT = 9411
      }

      resources {
        cpu    = 200
        memory = 100
      }
    }
  }
}