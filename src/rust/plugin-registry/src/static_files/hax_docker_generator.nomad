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

variable "rust_log" {
  type        = string
  description = "Controls the logging behavior of Rust-based services."
}

locals {
  # default values are cpu = 100 / mem = 300
  # per https://developer.hashicorp.com/nomad/docs/job-specification/resources
  consul_connect_proxy_cpu    = 50
  consul_connect_proxy_mem_mb = 50
}

job "grapl-plugin" {
  datacenters = ["dc1"]
  namespace   = "plugin-${var.plugin_id}"
  type        = "service"

  reschedule {
    # Make this a one-shot job
    attempts = 0
  }

  # This makes sure that generators only run on a certain subset of Nomad agents
  # that have "meta.is_grapl_plugin_host" set to true.
  # (We'll want to eventually ensure we have the opposite constraint on
  # non-plugin jobs.)
  # This is set in the Nomad agent's `client` stanza:
  # https://www.nomadproject.io/docs/configuration/client#meta
  constraint {
    attribute = "${meta.is_grapl_plugin_host}"
    value     = true
  }

  group "generator-execution-sidecar" {
    count = var.plugin_count

    network {
      mode = "bridge"
      port "generator-execution-sidecar" {}
    }

    service {
      name = "generator-exec-sidecar-${var.plugin_id}"
      tags = [
        "generator-execution-sidecar",
        "tenant-${var.tenant_id}",
        "plugin-${var.plugin_id}"
      ]

      connect {
        sidecar_task {
          resources {
            cpu    = local.consul_connect_proxy_cpu
            memory = local.consul_connect_proxy_mem_mb
          }
        }

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

    # The execution task pulls messages from the plugin-work-queue and
    # sends them to the generator
    task "generator-execution-sidecar" {
      driver = "docker"

      config {
        image = var.plugin_execution_sidecar_image
      }

      env {
        GENERATOR_EXECUTION_SIDECAR_PLUGIN_ID = var.plugin_id

        // The GENERATOR_CLIENT_ADDRESS is discovered at runtime because the
        // upstream's name is based on the plugin ID.
        GENERATOR_CLIENT_REQUEST_TIMEOUT               = "1s"
        GENERATOR_CLIENT_EXECUTOR_TIMEOUT              = "1s"
        GENERATOR_CLIENT_CONCURRENCY_LIMIT             = 16
        GENERATOR_CLIENT_INITIAL_BACKOFF_DELAY         = "10ms"
        GENERATOR_CLIENT_MAXIMUM_BACKOFF_DELAY         = "5s"
        GENERATOR_CLIENT_CONNECT_TIMEOUT               = "5s"
        GENERATOR_CLIENT_CONNECT_RETRIES               = 10
        GENERATOR_CLIENT_CONNECT_INITIAL_BACKOFF_DELAY = "1s"
        GENERATOR_CLIENT_CONNECT_MAXIMUM_BACKOFF_DELAY = "60s"

        PLUGIN_WORK_QUEUE_CLIENT_ADDRESS                       = "http://${NOMAD_UPSTREAM_ADDR_plugin-work-queue}"
        PLUGIN_WORK_QUEUE_CLIENT_REQUEST_TIMEOUT               = "1s"
        PLUGIN_WORK_QUEUE_CLIENT_EXECUTOR_TIMEOUT              = "1s"
        PLUGIN_WORK_QUEUE_CLIENT_CONCURRENCY_LIMIT             = 16
        PLUGIN_WORK_QUEUE_CLIENT_INITIAL_BACKOFF_DELAY         = "10ms"
        PLUGIN_WORK_QUEUE_CLIENT_MAXIMUM_BACKOFF_DELAY         = "5s"
        PLUGIN_WORK_QUEUE_CLIENT_CONNECT_TIMEOUT               = "5s"
        PLUGIN_WORK_QUEUE_CLIENT_CONNECT_RETRIES               = 10
        PLUGIN_WORK_QUEUE_CLIENT_CONNECT_INITIAL_BACKOFF_DELAY = "1s"
        PLUGIN_WORK_QUEUE_CLIENT_CONNECT_MAXIMUM_BACKOFF_DELAY = "60s"

        RUST_LOG       = var.rust_log
        RUST_BACKTRACE = 1
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
        sidecar_task {
          resources {
            cpu    = local.consul_connect_proxy_cpu
            memory = local.consul_connect_proxy_mem_mb
          }
        }

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
