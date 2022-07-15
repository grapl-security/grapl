variable "plugin_id" {
  type        = string
  description = "The ID for this plugin."
}

variable "tenant_id" {
  type        = string
  description = "The tenant's ID. Used in the plugin-execution-sidecar."
}

variable "plugin_artifact_url" {
  type        = string
  description = "The url that specifies which binary to run as the plugin."
}

variable "plugin_count" {
  type        = number
  default     = 1
  description = "The number of instances of the plugin to run."
}

variable "aws_account_id" {
  type        = string
  description = "The account ID of the aws account that holds onto the plugin binaries."
}

variable "plugin_runtime_image" {
  type        = string
  description = "The container that will load and run the plugin"
}

variable "plugin_execution_image" {
  type        = string
  description = "The container that will load and run the Generator Executor or Analyzer Executor"
}

variable "rust_log" {
  type        = string
  description = "Controls the logging behavior of Rust-based services."
}

variable "otel_exporter_jaeger_agent_host" {
  type        = string
  description = "Jaeger configuration"
}

variable "otel_exporter_jaeger_agent_port" {
  type        = number
  description = "Jaeger configuration"
}

job "grapl-plugin" {
  datacenters = ["dc1"]
  namespace   = "plugin-${var.plugin_id}"
  type        = "service"

  reschedule {
    # Make this a one-shot job
    attempts = 0
  }

  # We'll want to make sure we have the opposite constraint on other services
  # This is set in the Nomad agent's `client` stanza:
  # https://www.nomadproject.io/docs/configuration/client#meta
  constraint {
    attribute = "${meta.is_grapl_plugin_host}"
    value     = true
  }

  group "plugin-execution-sidecar" {
    count = var.plugin_count

    network {
      mode = "bridge"
      // TODO i think? possibly cargo culted?
      // dns {
      //   servers = local.dns_servers
      // }
      port "plugin-execution-sidecar" {}
    }

    service {
      name = "plugin-execution-sidecar-${var.plugin_id}"
      tags = [
        "serve_type:plugin-execution-sidecar",
        "tenant_id:${var.tenant_id}",
        "plugin_id:${var.plugin_id}"
      ]

      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "plugin-work-queue"
              # port unique but arbitrary - https://github.com/hashicorp/nomad/issues/7135
              local_bind_port = 1000
            }

            upstreams {
              destination_name = "plugin-${var.plugin_id}"
              # port unique but arbitrary - https://github.com/hashicorp/nomad/issues/7135
              local_bind_port = 1001
            }

            // TODO: upstream for graph-query-service
          }
        }
      }
    }

    # The execution task pulls messages from the plugin-work-queue and
    # sends them to the plugin
    task "plugin-execution-sidecar" {
      driver = "docker"

      config {
        image = var.plugin_execution_image
      }

      env {
        PLUGIN_EXECUTOR_PLUGIN_ID = "${var.plugin_id}"

        // FYI: the upstream plugin's address is discovered at runtime, not
        // env{}, because the upstream's name is based on ${PLUGIN_ID}.

        PLUGIN_WORK_QUEUE_CLIENT_ADDRESS = "http://${NOMAD_UPSTREAM_ADDR_plugin-work-queue}"

        RUST_LOG       = var.rust_log
        RUST_BACKTRACE = 1

        OTEL_EXPORTER_JAEGER_AGENT_HOST = var.otel_exporter_jaeger_agent_host
        OTEL_EXPORTER_JAEGER_AGENT_PORT = var.otel_exporter_jaeger_agent_port
      }
    }

    // TODO: task "tenant-plugin-graph-query-sidecar"

    restart {
      attempts = 1
      delay    = "5s"
    }
  }

  group "plugin" {
    network {
      mode = "bridge"
      // TODO
      // dns {
      //   servers = local.dns_servers
      // }
      port "plugin" {}
    }

    count = var.plugin_count

    service {
      name = "plugin-${var.plugin_id}"
      port = "plugin"
      tags = [
        "plugin",
        "tenant-${var.tenant_id}",
        "plugin-${var.plugin_id}"
      ]

      connect {
        sidecar_service {
        }
      }

      check {
        type     = "grpc"
        port     = "plugin"
        interval = "10s"
        timeout  = "3s"
      }
    }

    # a Docker task holding:
    # - the plugin binary itself (mounted)
    task "plugin" {
      driver = "docker"

      config {
        ports = ["plugin"]

        image      = var.plugin_runtime_image
        entrypoint = ["/bin/bash", "-o", "errexit", "-o", "nounset", "-c"]
        command = trimspace(<<EOF
        chmod +x "${PLUGIN_BIN}"
        "${PLUGIN_BIN}"
EOF
        )

        mount {
          type   = "bind"
          target = "/mnt/nomad_task_dir"
          source = "local/"
          # sigh - we need to `chmod +x` the binary, hence, mount is rw
          readonly = false
        }
      }

      artifact {
        source      = var.plugin_artifact_url
        destination = "local/plugin.bin"
        mode        = "file"
        headers {
          x-amz-expected-bucket-owner = var.aws_account_id
          x-amz-meta-client-id        = "nomad-deployer"
        }
      }

      env {
        TENANT_ID  = "${var.tenant_id}"
        PLUGIN_ID  = "${var.plugin_id}"
        PLUGIN_BIN = "/mnt/nomad_task_dir/plugin.bin"
        # Consumed by GeneratorServiceConfig
        PLUGIN_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_plugin}"

        OTEL_EXPORTER_JAEGER_AGENT_HOST = var.otel_exporter_jaeger_agent_host
        OTEL_EXPORTER_JAEGER_AGENT_PORT = var.otel_exporter_jaeger_agent_port

        # Should we make these eventually customizable?
        RUST_LOG       = var.rust_log
        RUST_BACKTRACE = 1
      }
    }

    restart {
      attempts = 1
      delay    = "5s"
    }
  }
}