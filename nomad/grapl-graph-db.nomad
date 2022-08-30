variable "rust_log" {
  type        = string
  description = "Controls the logging behavior of Rust-based services."
}

variable "container_images" {
  type        = map(string)
  description = <<EOF
  A map of $NAME_OF_TASK to the URL for that task's docker image ID.
  (See DockerImageId in Pulumi for further documentation.)
EOF
}

// TODO REMOVE????
variable "aws_env_vars_for_local" {
  type        = string
  description = <<EOF
With local-grapl, we have to inject:
- an endpoint
- an access key
- a secret key
With prod, these are all taken from the EC2 Instance Metadata in prod.
We have to provide a default value in prod; otherwise you can end up with a
weird nomad state parse error.
EOF
}

variable "observability_env_vars" {
  type        = string
  description = <<EOF
With local-grapl, we have to inject env vars for Opentelemetry.
In prod, this is currently disabled.
EOF
}

variable "graph_schema_manager_db" {
  type = object({
    hostname = string
    port     = number
    username = string
    password = string
  })
  description = "Vars for graph-schema-manager database"
}

variable "graph_db" {
  type = object({
    addresses = string
    username  = string
    password  = string
  })
  description = "Vars for graph (scylla) database"
}

variable "uid_allocator_db" {
  type = object({
    hostname = string
    port     = number
    username = string
    password = string
  })
  description = "Vars for uid-allocator database"
}

locals {
  dns_servers = [attr.unique.network.ip-address]
  # enabled
  rust_backtrace = 1
}

job "grapl-graph-db" {
  datacenters = ["dc1"]

  type = "service"

  # Specifies that this job is the most high priority job we have; nothing else should take precedence
  priority = 100

  update {
    # Automatically promotes to canaries if all canaries are healthy during an update / deployment
    auto_promote = true
    # Auto reverts to the last stable job variant if the update fails
    auto_revert = true
    # Spins up a "canary" instance of potentially destructive updates, validates that they are healthy, then promotes the instance to update
    canary       = 1
    max_parallel = 1
    # The min amount of reported "healthy" time before a instance is considered healthy and an allocation is opened up for further updates
    min_healthy_time = "15s"
  }

  group "scylla-provisioner" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "scylla-provisioner-port" {
      }
    }

    task "scylla-provisioner" {
      driver = "docker"

      config {
        image = var.container_images["scylla-provisioner"]
        ports = ["scylla-provisioner-port"]
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        SCYLLA_PROVISIONER_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_scylla-provisioner-port}"
        RUST_BACKTRACE                  = local.rust_backtrace
        RUST_LOG                        = var.rust_log
        GRAPH_DB_ADDRESSES              = var.graph_db.addresses
        GRAPH_DB_AUTH_PASSWORD          = var.graph_db.password
        GRAPH_DB_AUTH_USERNAME          = var.graph_db.username
      }
    }

    service {
      name = "scylla-provisioner"
      port = "scylla-provisioner-port"
      connect {
        sidecar_service {}
      }

      check {
        type     = "grpc"
        port     = "scylla-provisioner-port"
        interval = "10s"
        timeout  = "3s"
      }
    }
  }

  group "graph-query" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "graph-query-port" {
      }
    }

    task "graph-query" {
      driver = "docker"

      config {
        image = var.container_images["graph-query"]
        ports = ["graph-query-port"]
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        GRAPH_QUERY_SERVICE_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_graph-query-port}"
        RUST_BACKTRACE                   = local.rust_backtrace
        RUST_LOG                         = var.rust_log
        GRAPH_DB_ADDRESSES               = var.graph_db.addresses
        GRAPH_DB_AUTH_PASSWORD           = var.graph_db.password
        GRAPH_DB_AUTH_USERNAME           = var.graph_db.username
      }
    }

    service {
      name = "graph-query"
      port = "graph-query-port"
      connect {
        sidecar_service {}
      }

      check {
        type     = "grpc"
        port     = "graph-query-port"
        interval = "10s"
        timeout  = "3s"
      }
    }
  }

  group "graph-mutation" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "graph-mutation-port" {
      }
    }

    task "graph-mutation" {
      driver = "docker"

      config {
        image = var.container_images["graph-mutation"]
        ports = ["graph-mutation-port"]
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        GRAPH_MUTATION_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_graph-mutation-port}"

        RUST_BACKTRACE         = local.rust_backtrace
        RUST_LOG               = var.rust_log
        GRAPH_DB_ADDRESSES     = var.graph_db.addresses
        GRAPH_DB_AUTH_PASSWORD = var.graph_db.password
        GRAPH_DB_AUTH_USERNAME = var.graph_db.username

        # upstreams
        GRAPH_SCHEMA_MANAGER_CLIENT_ADDRESS = "http://${NOMAD_UPSTREAM_ADDR_graph-schema-manager}"
        UID_ALLOCATOR_CLIENT_ADDRESS        = "http://${NOMAD_UPSTREAM_ADDR_uid-allocator}"
      }
    }

    service {
      name = "graph-mutation"
      port = "graph-mutation-port"
      connect {
        sidecar_service {
          proxy {
            config {
              protocol = "grpc"
            }

            upstreams {
              destination_name = "graph-schema-manager"
              local_bind_port  = 1000
            }

            upstreams {
              destination_name = "uid-allocator"
              local_bind_port  = 1001
            }
          }
        }
      }

      check {
        type     = "grpc"
        port     = "graph-mutation-port"
        interval = "10s"
        timeout  = "3s"
      }
    }
  }

  group "uid-allocator" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }

      port "uid-allocator-port" {
      }
    }

    task "uid-allocator" {
      driver = "docker"

      config {
        image = var.container_images["uid-allocator"]
        ports = ["uid-allocator-port"]
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        UID_ALLOCATOR_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_uid-allocator-port}"

        COUNTER_DB_ADDRESS  = "${var.uid_allocator_db.hostname}:${var.uid_allocator_db.port}"
        COUNTER_DB_PASSWORD = var.uid_allocator_db.password
        COUNTER_DB_USERNAME = var.uid_allocator_db.username

        DEFAULT_ALLOCATION_SIZE = 100
        PREALLOCATION_SIZE      = 10000
        MAXIMUM_ALLOCATION_SIZE = 1000
        RUST_BACKTRACE          = local.rust_backtrace
        RUST_LOG                = var.rust_log
      }
    }

    service {
      name = "uid-allocator"
      port = "uid-allocator-port"
      connect {
        sidecar_service {
        }
      }

      check {
        type     = "grpc"
        port     = "uid-allocator-port"
        interval = "10s"
        timeout  = "3s"
      }
    }
  }

  group "graph-schema-manager" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "graph-schema-manager-port" {
      }
    }

    task "graph-schema-manager" {
      driver = "docker"

      config {
        image = var.container_images["graph-schema-manager"]
        ports = ["graph-schema-manager-port"]
      }

      // TODO remove??
      template {
        data        = var.aws_env_vars_for_local
        destination = "aws_vars.env"
        env         = true
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log

        GRAPH_SCHEMA_MANAGER_BIND_ADDRESS                    = "0.0.0.0:${NOMAD_PORT_graph-schema-manager-port}"
        GRAPH_SCHEMA_MANAGER_HEALTHCHECK_POLLING_INTERVAL_MS = 5000

        GRAPH_SCHEMA_DB_ADDRESS  = "${var.graph_schema_manager_db.hostname}:${var.graph_schema_manager_db.port}"
        GRAPH_SCHEMA_DB_PASSWORD = var.graph_schema_manager_db.password
        GRAPH_SCHEMA_DB_USERNAME = var.graph_schema_manager_db.username
      }
    }

    service {
      name = "graph-schema-manager"
      port = "graph-schema-manager-port"
      connect {
        sidecar_service {
        }
      }

      check {
        type     = "grpc"
        port     = "graph-schema-manager-port"
        interval = "10s"
        timeout  = "3s"
      }
    }
  }

}
