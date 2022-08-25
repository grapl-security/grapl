variable "rust_log" {
  type        = string
  description = "Controls the logging behavior of Rust-based services."
}

variable "py_log_level" {
  type        = string
  description = "Controls the logging behavior of Python-based services."
}

variable "container_images" {
  type        = map(string)
  description = <<EOF
  A map of $NAME_OF_TASK to the URL for that task's docker image ID.
  (See DockerImageId in Pulumi for further documentation.)
EOF
}

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

variable "graph_db" {
  type = object({
    addresses = string
    username  = string
    password  = string
  })
  description = "Vars for graph (scylla) database"
}

variable "observability_env_vars" {
  type        = string
  description = <<EOF
With local-grapl, we have to inject env vars for Opentelemetry.
In prod, this is currently disabled.
EOF
}

variable "aws_region" {
  type = string
}

variable "dgraph_replicas" {
  type    = number
  default = 1
  validation {
    condition     = var.dgraph_replicas % 2 == 1
    error_message = "This value must be odd. Otherwise dgraph_zero will exit."
  }
}

variable "dgraph_shards" {
  type    = number
  default = 1
}

variable "kafka_bootstrap_servers" {
  type        = string
  description = "The URL(s) (possibly comma-separated) of the Kafka bootstrap servers."
}

variable "schema_table_name" {
  type        = string
  description = "What is the name of the schema table?"
}

variable "schema_properties_table_name" {
  type        = string
  description = "What is the name of the schema properties table?"
}

# https://github.com/grapl-security/grapl/blob/af6f2c197d52e9941047aab813c30d2cbfd54523/pulumi/infra/dynamodb.py#L118
variable "session_table_name" {
  type        = string
  description = "What is the name of the session table?"
}

variable "plugin_registry_db" {
  type = object({
    hostname = string
    port     = number
    username = string
    password = string
  })
  description = "Vars for plugin-registry database"
}

variable "plugin_registry_kernel_artifact_url" {
  type        = string
  description = "URL specifying the kernel.tar.gz for the Firecracker VM"
}

variable "plugin_registry_rootfs_artifact_url" {
  type        = string
  description = "URL specifying the rootfs.tar.gz for the Firecracker VM"
}

variable "organization_management_db" {
  type = object({
    hostname = string
    port     = number
    username = string
    password = string
  })
  description = "Vars for organization-management database"
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

variable "organization_management_healthcheck_polling_interval_ms" {
  type        = string
  description = "The amount of time to wait between each healthcheck execution."
}

variable "pipeline_ingress_healthcheck_polling_interval_ms" {
  type        = string
  description = "The amount of time to wait between each healthcheck execution."
}

variable "kafka_credentials" {
  description = "Map from service-name to kafka credentials for that service"
  type = map(object({
    # The username to authenticate with Confluent Cloud cluster.
    sasl_username = string
    # The password to authenticate with Confluent Cloud cluster.
    sasl_password = string
  }))
}

variable "kafka_consumer_groups" {
  description = "Map from service-name to the consumer group for that service to join"
  type        = map(string)
}

variable "plugin_work_queue_db" {
  type = object({
    hostname = string
    port     = number
    username = string
    password = string
  })
  description = "Vars for plugin-work-queue database"
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

variable "event_source_db" {
  type = object({
    hostname = string
    port     = number
    username = string
    password = string
  })
  description = "Vars for event-source database"
}

variable "plugin_registry_bucket_aws_account_id" {
  type        = string
  description = "The account id that owns the bucket where plugins are stored"
}

variable "plugin_registry_bucket_name" {
  type        = string
  description = "The name of the bucket where plugins are stored"
}

variable "test_user_name" {
  type        = string
  description = "The name of the test user"
}

variable "user_auth_table" {
  type        = string
  description = "What is the name of the DynamoDB user auth table?"
}

variable "user_session_table" {
  type        = string
  description = "What is the name of the DynamoDB user session table?"
}

variable "google_client_id" {
  type        = string
  description = "Google client ID used for authenticating web users via Sign In With Google"
}

locals {
  dgraph_zero_grpc_private_port_base  = 5080
  dgraph_zero_http_private_port_base  = 6080
  dgraph_alpha_grpc_private_port_base = 7080
  dgraph_alpha_http_private_port_base = 8080
  dgraph_alpha_grpc_public_port_base  = 9080

  # DGraph Alphas (shards * replicas)
  dgraph_alphas = [for alpha_id in range(0, var.dgraph_replicas * var.dgraph_shards) : {
    id : alpha_id,
    grpc_private_port : local.dgraph_alpha_grpc_private_port_base + alpha_id,
    grpc_public_port : local.dgraph_alpha_grpc_public_port_base + alpha_id,
    http_port : local.dgraph_alpha_http_private_port_base + alpha_id
  }]

  # DGraph Zeros (replicas)
  dgraph_zeros = [for zero_id in range(1, var.dgraph_replicas) : {
    id : zero_id,
    grpc_private_port : local.dgraph_zero_grpc_private_port_base + zero_id,
    http_port : local.dgraph_zero_http_private_port_base + zero_id,
  }]

  # String that contains all of the Zeros for the Alphas to talk to and ensure they don't go down when one dies
  zero_alpha_connect_str = join(",", [for zero_id in range(0, var.dgraph_replicas) : "localhost:${local.dgraph_zero_grpc_private_port_base + zero_id}"])

  # String that contains all of the running Alphas for clients connecting to Dgraph (so they can do loadbalancing)
  alpha_grpc_connect_str = join(",", [for alpha in local.dgraph_alphas : "localhost:${alpha.grpc_public_port}"])

  dgraph_volume_args = {
    target = "/dgraph"
    source = "grapl-data-dgraph"
  }

  dns_servers = [attr.unique.network.ip-address]

  # Grapl services
  graphql_endpoint_port = 5000

  # enabled
  rust_backtrace = 1
}

job "grapl-core" {
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

  group "dgraph-zero-0" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
    }

    task "dgraph-zero" {
      driver = "docker"

      config {
        image = var.container_images["dgraph"]
        args = [
          "dgraph",
          "zero",
          "--my", "localhost:${local.dgraph_zero_grpc_private_port_base}",
          "--replicas", "${var.dgraph_replicas}",
          "--raft", "idx=1",
        ]

        mount {
          type     = "volume"
          target   = "${local.dgraph_volume_args.target}"
          source   = "${local.dgraph_volume_args.source}"
          readonly = false
        }
      }
    }

    service {
      name = "dgraph-zero-0-grpc-private"
      port = "${local.dgraph_zero_grpc_private_port_base}"
      tags = ["dgraph", "zero", "grpc"]

      connect {
        sidecar_service {
          proxy {
            # Connect the Zero leader to the Zero followers
            dynamic "upstreams" {
              iterator = zero_follower
              for_each = local.dgraph_zeros

              content {
                destination_name = "dgraph-zero-${zero_follower.value.id}-grpc-private"
                local_bind_port  = zero_follower.value.grpc_private_port
              }
            }

            # Connect this Zero leader to the Alphas
            dynamic "upstreams" {
              iterator = alpha
              for_each = [for alpha in local.dgraph_alphas : alpha]

              content {
                destination_name = "dgraph-alpha-${alpha.value.id}-grpc-private"
                local_bind_port  = alpha.value.grpc_private_port
              }
            }
          }
        }
      }
    }
  }

  # Create DGraph Zero followers
  dynamic "group" {
    iterator = zero
    for_each = local.dgraph_zeros
    labels   = ["dgraph-zero-${zero.value.id}"]

    content {
      network {
        mode = "bridge"
        dns {
          servers = local.dns_servers
        }
        port "healthcheck" {
          to = -1
        }
      }

      task "dgraph-zero" {
        driver = "docker"

        config {
          image = var.container_images["dgraph"]
          args = [
            "dgraph",
            "zero",
            "--my", "localhost:${zero.value.grpc_private_port}",
            "--replicas", "${var.dgraph_replicas}",
            "--raft", "idx=${zero.value.id + 1}",
            "--port_offset", "${zero.value.id}",
            "--peer", "localhost:${local.dgraph_zero_grpc_private_port_base}"
          ]

          mount {
            type     = "volume"
            target   = "${local.dgraph_volume_args.target}"
            source   = "${local.dgraph_volume_args.source}"
            readonly = false
          }
        }
      }
      service {
        name = "dgraph-zero-${zero.value.id}-grpc-private"
        port = "${zero.value.grpc_private_port}"
        tags = ["dgraph", "zero", "grpc"]

        connect {
          sidecar_service {
            proxy {
              # Connect to the Zero leader
              upstreams {
                destination_name = "dgraph-zero-0-grpc-private"
                local_bind_port  = local.dgraph_zero_grpc_private_port_base
              }

              # Connect this Zero follower to other Zero followers (but not to itself, obviously)
              dynamic "upstreams" {
                iterator = zero_follower
                for_each = [for zero_follower in local.dgraph_zeros : zero_follower if zero_follower.id != zero.value.id]

                content {
                  destination_name = "dgraph-zero-${zero_follower.value.id}-grpc-private"
                  local_bind_port  = zero_follower.value.grpc_private_port
                }
              }

              # Connect this Zero follower to the Alphas
              dynamic "upstreams" {
                iterator = alpha
                for_each = [for alpha in local.dgraph_alphas : alpha]

                content {
                  destination_name = "dgraph-alpha-${alpha.value.id}-grpc-private"
                  local_bind_port  = alpha.value.grpc_private_port
                }
              }

              # We need to expose the health check for consul to be able to reach it
              expose {
                path {
                  path            = "/health"
                  protocol        = "http"
                  local_path_port = zero.value.http_port
                  listener_port   = "healthcheck"
                }
              }

            }
          }
        }

        check {
          type     = "http"
          name     = "dgraph-zero-http-healthcheck"
          path     = "/health"
          port     = "healthcheck"
          method   = "GET"
          interval = "30s"
          timeout  = "5s"

          check_restart {
            limit           = 3
            grace           = "30s"
            ignore_warnings = false
          }
        }
      }
    }
  }

  # Create DGraph Alphas
  dynamic "group" {
    iterator = alpha
    for_each = local.dgraph_alphas
    labels   = ["dgraph-alpha-${alpha.value.id}"]

    content {
      network {
        mode = "bridge"
        dns {
          servers = local.dns_servers
        }
        port "healthcheck" {
          to = -1
        }
        port "dgraph-alpha-port" {
          # Primarily here to let us use ratel.
          # Could be potentially replaced with a gateway stanza or something.
          to = alpha.value.http_port
        }
      }

      task "dgraph-alpha" {
        driver = "docker"

        config {
          image = var.container_images["dgraph"]
          args = [
            "dgraph",
            "alpha",
            "--my", "localhost:${alpha.value.grpc_private_port}",
            "--port_offset", "${alpha.value.id}",
            "--zero", "${local.zero_alpha_connect_str}"
          ]

          mount {
            type     = "volume"
            target   = "${local.dgraph_volume_args.target}"
            source   = "${local.dgraph_volume_args.source}"
            readonly = false
          }

          ports = ["dgraph-alpha-port"]
        }

        resources {
          memory = 512
        }

      }

      service {
        name = "dgraph-alpha-${alpha.value.id}-grpc-private"
        port = "${alpha.value.grpc_private_port}"
        tags = ["dgraph", "alpha", "grpc"]

        connect {
          sidecar_service {
            proxy {
              # Connect to the Zero leader
              upstreams {
                destination_name = "dgraph-zero-0-grpc-private"
                local_bind_port  = local.dgraph_zero_grpc_private_port_base
              }

              # Connect this Alpha to Zero followers
              dynamic "upstreams" {
                iterator = zero_follower
                for_each = [for zero_follower in local.dgraph_zeros : zero_follower]

                content {
                  destination_name = "dgraph-zero-${zero_follower.value.id}-grpc-private"
                  local_bind_port  = zero_follower.value.grpc_private_port
                }
              }

              # Connect this Alpha to Other Alphas (but not to itself, obviously)
              dynamic "upstreams" {
                iterator = alpha_peer
                for_each = [for alpha_peer in local.dgraph_alphas : alpha_peer if alpha_peer.id != alpha.value.id]

                content {
                  destination_name = "dgraph-alpha-${alpha_peer.value.id}-grpc-private"
                  local_bind_port  = alpha_peer.value.grpc_private_port
                }
              }
            }
          }
        }
      }

      service {
        name = "dgraph-alpha-${alpha.value.id}-grpc-public"
        port = "${alpha.value.grpc_public_port}"
        tags = ["dgraph", "alpha", "grpc"]

        connect {
          sidecar_service {}
        }
      }

      service {
        name = "dgraph-alpha-${alpha.value.id}-http"
        port = "${alpha.value.http_port}"
        tags = ["dgraph", "alpha", "http"]

        connect {
          sidecar_service {
            proxy {
              config {
                protocol = "http"
              }

              # We need to expose the health check for consul to be able to reach it
              expose {
                path {
                  path            = "/health"
                  protocol        = "http"
                  local_path_port = alpha.value.http_port
                  listener_port   = "healthcheck"
                }
              }
            }
          }
        }

        check {
          type     = "http"
          name     = "dgraph-alpha-http-healthcheck"
          path     = "/health"
          port     = "healthcheck"
          method   = "GET"
          interval = "30s"
          timeout  = "5s"

          check_restart {
            limit           = 3
            grace           = "30s"
            ignore_warnings = false
          }
        }
      }
    }
  }

  #######################################
  ## Begin actual Grapl core services ##
  #######################################

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

  group "generator-dispatcher" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
    }

    task "generator-dispatcher" {
      driver = "docker"

      config {
        image = var.container_images["generator-dispatcher"]
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        # Upstreams
        PLUGIN_WORK_QUEUE_CLIENT_ADDRESS = "http://${NOMAD_UPSTREAM_ADDR_plugin-work-queue}"
        PLUGIN_REGISTRY_CLIENT_ADDRESS   = "http://${NOMAD_UPSTREAM_ADDR_plugin-registry}"

        # Kafka
        KAFKA_BOOTSTRAP_SERVERS   = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME       = var.kafka_credentials["generator-dispatcher"].sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials["generator-dispatcher"].sasl_password
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_groups["generator-dispatcher"]
        KAFKA_CONSUMER_TOPIC      = "raw-logs"
        KAFKA_PRODUCER_TOPIC      = "generated-graphs"
        KAFKA_RETRY_TOPIC         = "raw-logs-retry"

        # TODO: should equal number of raw-logs partitions
        WORKER_POOL_SIZE = 10

        GENERATOR_IDS_CACHE_CAPACITY            = 10000
        GENERATOR_IDS_CACHE_TTL_MS              = 5000
        GENERATOR_IDS_CACHE_UPDATER_POOL_SIZE   = 10
        GENERATOR_IDS_CACHE_UPDATER_QUEUE_DEPTH = 1000

        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log
      }
    }

    service {
      name = "generator-dispatcher"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "plugin-work-queue"
              local_bind_port  = 1000
            }
            upstreams {
              destination_name = "plugin-registry"
              local_bind_port  = 1001
            }
          }
        }
      }
    }
  }

  group "graph-merger" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
    }

    task "graph-merger" {
      driver = "docker"

      config {
        image = var.container_images["graph-merger"]
      }

      # This writes an env file that gets read by nomad automatically
      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        AWS_REGION         = var.aws_region
        RUST_LOG           = var.rust_log
        RUST_BACKTRACE     = local.rust_backtrace
        MG_ALPHAS          = local.alpha_grpc_connect_str
        GRAPL_SCHEMA_TABLE = var.schema_table_name

        KAFKA_BOOTSTRAP_SERVERS   = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME       = var.kafka_credentials["graph-merger"].sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials["graph-merger"].sasl_password
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_groups["graph-merger"]
        KAFKA_CONSUMER_TOPIC      = "identified-graphs"
        KAFKA_PRODUCER_TOPIC      = "merged-graphs"
      }
    }

    service {
      name = "graph-merger"

      connect {
        sidecar_service {
          proxy {
            dynamic "upstreams" {
              iterator = alpha
              for_each = local.dgraph_alphas

              content {
                destination_name = "dgraph-alpha-${alpha.value.id}-grpc-public"
                local_bind_port  = alpha.value.grpc_public_port
              }
            }
          }
        }
      }
    }
  }

  group "node-identifier" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
    }

    task "node-identifier" {
      driver = "docker"

      config {
        image = var.container_images["node-identifier"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        AWS_REGION = var.aws_region

        RUST_LOG       = var.rust_log
        RUST_BACKTRACE = local.rust_backtrace

        KAFKA_BOOTSTRAP_SERVERS   = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME       = var.kafka_credentials["node-identifier"].sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials["node-identifier"].sasl_password
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_groups["node-identifier"]
        KAFKA_CONSUMER_TOPIC      = "generated-graphs"
        KAFKA_PRODUCER_TOPIC      = "identified-graphs"

        GRAPL_SCHEMA_TABLE          = var.schema_table_name
        GRAPL_DYNAMIC_SESSION_TABLE = var.session_table_name
      }

      service {
        name = "node-identifier"
      }
    }
  }

  group "graphql-endpoint" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "graphql-endpoint-port" {}
    }

    task "graphql-endpoint" {
      driver = "docker"

      config {
        image = var.container_images["graphql-endpoint"]
        ports = ["graphql-endpoint-port"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        RUST_LOG = var.rust_log
        # JS SDK only recognized AWS_REGION whereas rust and python SDKs use DEFAULT_AWS_REGION
        AWS_REGION                    = var.aws_region
        MG_ALPHAS                     = local.alpha_grpc_connect_str
        GRAPL_SCHEMA_TABLE            = var.schema_table_name
        GRAPL_SCHEMA_PROPERTIES_TABLE = var.schema_properties_table_name
        IS_LOCAL                      = "True"
        JWT_SECRET_ID                 = "JWT_SECRET_ID"
        PORT                          = "${NOMAD_PORT_graphql-endpoint-port}"
      }
    }

    service {
      name = "graphql-endpoint"
      port = "graphql-endpoint-port"

      connect {
        sidecar_service {
          proxy {
            dynamic "upstreams" {
              iterator = alpha
              for_each = local.dgraph_alphas

              content {
                destination_name = "dgraph-alpha-${alpha.value.id}-grpc-public"
                local_bind_port  = alpha.value.grpc_public_port
              }
            }
          }
        }
      }
    }
  }

  group "kafka-retry" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
    }

    task "generator-dispatcher-retry" {
      driver = "docker"

      config {
        image = var.container_images["kafka-retry"]
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        # Kafka
        KAFKA_BOOTSTRAP_SERVERS   = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME       = var.kafka_credentials["generator-dispatcher-retry"].sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials["generator-dispatcher-retry"].sasl_password
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_groups["generator-dispatcher-retry"]
        KAFKA_RETRY_TOPIC         = "raw-logs-retry"
        KAFKA_RETRY_DELAY_MS      = 500
        KAFKA_PRODUCER_TOPIC      = "raw-logs"

        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log
      }

      service {
        name = "generator-dispatcher-retry"
      }
    }
  }

  group "web-ui" {
    count = 1

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }

      port "web-ui-port" {
      }
    }

    task "web-ui" {
      driver = "docker"

      config {
        image = var.container_images["web-ui"]
        ports = ["web-ui-port"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        # For the DynamoDB client
        AWS_REGION = var.aws_region

        GRAPL_USER_AUTH_TABLE    = var.user_auth_table
        GRAPL_USER_SESSION_TABLE = var.user_session_table

        GRAPL_WEB_UI_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_web-ui-port}"
        GRAPL_GRAPHQL_ENDPOINT    = "http://${NOMAD_UPSTREAM_ADDR_graphql-endpoint}"
        GRAPL_GOOGLE_CLIENT_ID    = var.google_client_id
        RUST_LOG                  = var.rust_log
        RUST_BACKTRACE            = local.rust_backtrace
      }

      resources {
        # Can OOM with default limit of 300 MB and a number of connections at the same
        # time, which seems low, but is reliably produced with the its integration tests.
        memory = 1024
      }
    }

    service {
      name = "web-ui"
      port = "web-ui-port"
      connect {
        sidecar_service {
          proxy {
            config {
              protocol = "http"
            }
            upstreams {
              destination_name = "graphql-endpoint"
              local_bind_port  = local.graphql_endpoint_port
            }
          }
        }
      }
      check {
        type     = "http"
        name     = "grapl-web-health"
        path     = "/api/health"
        interval = "10s"
        timeout  = "2s"
      }
    }
  }

  group "organization-management" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "organization-management-port" {
      }
    }

    task "organization-management" {
      driver = "docker"

      config {
        image = var.container_images["organization-management"]
        ports = ["organization-management-port"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "organization-management.env"
        env         = true
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        AWS_REGION                           = var.aws_region
        NOMAD_SERVICE_ADDRESS                = "${attr.unique.network.ip-address}:4646"
        ORGANIZATION_MANAGEMENT_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_organization-management-port}"
        RUST_BACKTRACE                       = local.rust_backtrace
        RUST_LOG                             = var.rust_log
        ORGANIZATION_MANAGEMENT_DB_ADDRESS   = "${var.organization_management_db.hostname}:${var.organization_management_db.port}"
        ORGANIZATION_MANAGEMENT_DB_PASSWORD  = var.organization_management_db.password
        ORGANIZATION_MANAGEMENT_DB_USERNAME  = var.organization_management_db.username

        ORGANIZATION_MANAGEMENT_HEALTHCHECK_POLLING_INTERVAL_MS = var.organization_management_healthcheck_polling_interval_ms
      }
    }

    service {
      name = "organization-management"
      port = "organization-management-port"
      connect {
        sidecar_service {
        }
      }
    }
  }

  group "pipeline-ingress" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "pipeline-ingress-port" {
      }
    }

    task "pipeline-ingress" {
      driver = "docker"

      config {
        image = var.container_images["pipeline-ingress"]
        ports = ["pipeline-ingress-port"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "pipeline-ingress-env"
        env         = true
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        AWS_REGION                                       = var.aws_region
        NOMAD_SERVICE_ADDRESS                            = "${attr.unique.network.ip-address}:4646"
        PIPELINE_INGRESS_BIND_ADDRESS                    = "0.0.0.0:${NOMAD_PORT_pipeline-ingress-port}"
        RUST_BACKTRACE                                   = local.rust_backtrace
        RUST_LOG                                         = var.rust_log
        PIPELINE_INGRESS_HEALTHCHECK_POLLING_INTERVAL_MS = var.pipeline_ingress_healthcheck_polling_interval_ms
        KAFKA_BOOTSTRAP_SERVERS                          = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME                              = var.kafka_credentials["pipeline-ingress"].sasl_username
        KAFKA_SASL_PASSWORD                              = var.kafka_credentials["pipeline-ingress"].sasl_password
        KAFKA_PRODUCER_TOPIC                             = "raw-logs"
      }
    }

    service {
      name = "pipeline-ingress"
      port = "pipeline-ingress-port"
      connect {
        sidecar_service {
        }
      }

      check {
        type     = "grpc"
        port     = "pipeline-ingress-port"
        interval = "10s"
        timeout  = "3s"
      }
    }
  }

  group "plugin-registry" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }

      port "plugin-registry-port" {
      }
    }

    task "plugin-registry" {
      driver = "docker"

      config {
        image = var.container_images["plugin-registry"]
        ports = ["plugin-registry-port"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        AWS_REGION                                      = var.aws_region
        NOMAD_SERVICE_ADDRESS                           = "${attr.unique.network.ip-address}:4646"
        PLUGIN_REGISTRY_BIND_ADDRESS                    = "0.0.0.0:${NOMAD_PORT_plugin-registry-port}"
        PLUGIN_REGISTRY_DB_ADDRESS                      = "${var.plugin_registry_db.hostname}:${var.plugin_registry_db.port}"
        PLUGIN_REGISTRY_DB_PASSWORD                     = var.plugin_registry_db.password
        PLUGIN_REGISTRY_DB_USERNAME                     = var.plugin_registry_db.username
        PLUGIN_BOOTSTRAP_CONTAINER_IMAGE                = var.container_images["plugin-bootstrap"]
        PLUGIN_REGISTRY_KERNEL_ARTIFACT_URL             = var.plugin_registry_kernel_artifact_url
        PLUGIN_REGISTRY_ROOTFS_ARTIFACT_URL             = var.plugin_registry_rootfs_artifact_url
        PLUGIN_REGISTRY_HAX_DOCKER_PLUGIN_RUNTIME_IMAGE = var.container_images["hax-docker-plugin-runtime"]
        PLUGIN_REGISTRY_BUCKET_AWS_ACCOUNT_ID           = var.plugin_registry_bucket_aws_account_id
        PLUGIN_REGISTRY_BUCKET_NAME                     = var.plugin_registry_bucket_name
        PLUGIN_EXECUTION_OBSERVABILITY_ENV_VARS         = var.observability_env_vars
        PLUGIN_EXECUTION_GENERATOR_SIDECAR_IMAGE        = var.container_images["generator-execution-sidecar"]
        PLUGIN_EXECUTION_ANALYZER_SIDECAR_IMAGE         = var.container_images["analyzer-execution-sidecar"]

        # common Rust env vars
        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log
      }

      resources {
        # Probably too much. Let's figure out buffered writes to s3
        memory = 512
      }
    }

    service {
      name = "plugin-registry"
      port = "plugin-registry-port"
      connect {
        sidecar_service {
        }
      }

      check {
        type     = "grpc"
        port     = "plugin-registry-port"
        interval = "10s"
        timeout  = "3s"
      }
    }
  }

  group "plugin-work-queue" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }

      port "plugin-work-queue-port" {
      }
    }

    task "plugin-work-queue" {
      driver = "docker"

      config {
        image = var.container_images["plugin-work-queue"]
        ports = ["plugin-work-queue-port"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        PLUGIN_WORK_QUEUE_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_plugin-work-queue-port}"
        PLUGIN_WORK_QUEUE_DB_ADDRESS   = "${var.plugin_work_queue_db.hostname}:${var.plugin_work_queue_db.port}"
        PLUGIN_WORK_QUEUE_DB_PASSWORD  = var.plugin_work_queue_db.password
        PLUGIN_WORK_QUEUE_DB_USERNAME  = var.plugin_work_queue_db.username
        # Hardcoded, but makes little sense to pipe up through Pulumi
        PLUGIN_WORK_QUEUE_HEALTHCHECK_POLLING_INTERVAL_MS = 5000

        KAFKA_BOOTSTRAP_SERVERS = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME     = var.kafka_credentials["plugin-work-queue"].sasl_username
        KAFKA_SASL_PASSWORD     = var.kafka_credentials["plugin-work-queue"].sasl_password

        GENERATOR_KAFKA_PRODUCER_TOPIC = "generated-graphs"
        # ANALYZER_KAFKA_PRODUCER_TOPIC = "TODO"

        # common Rust env vars
        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log
      }
    }

    service {
      name = "plugin-work-queue"
      port = "plugin-work-queue-port"
      connect {
        sidecar_service {
        }
      }

      check {
        type     = "grpc"
        port     = "plugin-work-queue-port"
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
        UID_ALLOCATOR_DB_HOSTNAME  = var.uid_allocator_db.hostname
        UID_ALLOCATOR_DB_PASSWORD  = var.uid_allocator_db.password
        UID_ALLOCATOR_DB_PORT      = var.uid_allocator_db.port
        UID_ALLOCATOR_DB_USERNAME  = var.uid_allocator_db.username
        RUST_BACKTRACE             = local.rust_backtrace
        RUST_LOG                   = var.rust_log
      }
    }

    service {
      name = "uid-allocator"
      port = "uid-allocator-port"
      connect {
        sidecar_service {
        }
      }
    }
  }

  group "event-source" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }

      port "event-source-port" {
      }
    }

    task "event-source" {
      driver = "docker"

      config {
        image = var.container_images["event-source"]
        ports = ["event-source-port"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        EVENT_SOURCE_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_event-source-port}"
        EVENT_SOURCE_DB_ADDRESS   = "${var.event_source_db.hostname}:${var.event_source_db.port}"
        EVENT_SOURCE_DB_PASSWORD  = var.event_source_db.password
        EVENT_SOURCE_DB_USERNAME  = var.event_source_db.username
        # Hardcoded, but makes little sense to pipe up through Pulumi
        EVENT_SOURCE_HEALTHCHECK_POLLING_INTERVAL_MS = 5000

        # common Rust env vars
        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log
      }
    }

    service {
      name = "event-source"
      port = "event-source-port"
      connect {
        sidecar_service {
        }
      }

      check {
        type     = "grpc"
        port     = "event-source-port"
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
