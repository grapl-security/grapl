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

variable "plugin_execution_sidecar_image" {
  type        = string
  description = "The container that will load and run the Generator Executor or Analyzer Executor"
}

variable "graph_query_proxy_image" {
  description = "Container image for running Graph Query Proxy. Only specify this for Analyzers."
  type        = string
  default     = null
}

variable "rust_log" {
  type        = string
  description = "Controls the logging behavior of Rust-based services."
}

variable "observability_env_vars" {
  type        = string
  description = <<EOF
With local-grapl, we have to inject env vars for Opentelemetry.
In prod, this is currently disabled.
EOF
}

job "grapl-plugin" {
  datacenters = ["dc1"]
  namespace   = "plugin-${var.plugin_id}"
  type        = "service"

  reschedule {
    # Make this a one-shot job
    attempts = 0
  }

  # This makes sure that analyzers only run on a certain subset of Nomad agents
  # that have "meta.is_grapl_plugin_host" set to true.
  # (We'll want to eventually ensure we have the opposite constraint on 
  # non-plugin jobs.)
  # This is set in the Nomad agent's `client` stanza:
  # https://www.nomadproject.io/docs/configuration/client#meta
  constraint {
    attribute = "${meta.is_grapl_plugin_host}"
    value     = true
  }

  group "analyzer-execution-sidecar" {
    count = var.plugin_count

    network {
      mode = "bridge"
      port "analyzer-execution-sidecar" {}
      port "graph-query-proxy" {}
    }

    service {
      name = "analyzer-exec-sidecar-${var.plugin_id}"
      tags = [
        "analyzer-execution-sidecar",
        "tenant-${var.tenant_id}",
        "plugin-${var.plugin_id}"
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
          }
        }
      }
    }

    service {
      name = "graph-query-proxy-${var.plugin_id}"
      port = "graph-query-proxy"
      tags = [
        "graph-query-proxy",
        "tenant-${var.tenant_id}",
        "plugin-${var.plugin_id}"
      ]

      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "graph-query"
              # port unique but arbitrary - https://github.com/hashicorp/nomad/issues/7135
              local_bind_port = 1000
            }
          }
        }
      }

      check {
        type     = "grpc"
        port     = "graph-query-proxy"
        interval = "10s"
        timeout  = "3s"
      }
    }

    # The execution task pulls messages from the plugin-work-queue and
    # sends them to the analyzer
    task "analyzer-execution-sidecar" {
      driver = "docker"

      config {
        image = var.plugin_execution_sidecar_image
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        PLUGIN_EXECUTOR_PLUGIN_ID = var.plugin_id

        // FYI: the upstream plugin's address is discovered at runtime, not
        // env{}, because the upstream's name is based on ${PLUGIN_ID}.

        PLUGIN_WORK_QUEUE_CLIENT_ADDRESS = "http://${NOMAD_UPSTREAM_ADDR_plugin-work-queue}"

        RUST_LOG       = var.rust_log
        RUST_BACKTRACE = 1
      }
    }

    # A level of indirection in front of the Graph Query Service that prevents
    # the plugin from querying other tenants' data.

    task "graph-query-proxy-sidecar" {
      driver = "docker"

      config {
        image = var.graph_query_proxy_image
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        GRAPH_QUERY_PROXY_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_graph-query-proxy}"
        GRAPH_QUERY_CLIENT_ADDRESS     = "http://${NOMAD_UPSTREAM_ADDR_graph-query}"
        TENANT_ID                      = var.tenant_id

        RUST_LOG       = var.rust_log
        RUST_BACKTRACE = 1
      }

      resources {
        cpu    = 25  // MHz
        memory = 128 // MB
      }
    }

    restart {
      attempts = 1
      delay    = "5s"
    }
  }

  group "plugin" {
    network {
      mode = "bridge"
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
          proxy {
            upstreams {
              destination_name = "graph-query-proxy-${var.plugin_id}"
              local_bind_port  = 1000
            }
          }
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

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        TENANT_ID  = "${var.tenant_id}"
        PLUGIN_ID  = "${var.plugin_id}"
        PLUGIN_BIN = "/mnt/nomad_task_dir/plugin.bin"
        # Consumed by GeneratorServiceConfig
        PLUGIN_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_plugin}"

        # We have to do some Bash variable indirection - {!VAR_NAME}, 
        # since the upstream has a variable in its name.
        GRAPH_QUERY_UPSTREAM_ADDR  = "NOMAD_UPSTREAM_ADDR_graph-query-sidecar-${var.plugin_id}"
        GRAPH_QUERY_CLIENT_ADDRESS = "http://${!GRAPH_QUERY_UPSTREAM_ADDR}"

        RUST_LOG       = var.rust_log
        RUST_BACKTRACE = 1
      }

      // Each plugin should ideally have a very small footprint.
      resources {
        cpu    = 25  // MHz
        memory = 128 // MB
      }
    }

    restart {
      attempts = 1
      delay    = "5s"
    }
  }
}
