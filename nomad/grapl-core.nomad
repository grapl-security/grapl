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

variable "aws_region" {
  type = string
}

variable "analyzer_bucket" {
  type        = string
  description = "The s3 bucket which the analyzer stores items to analyze"
}

variable "analyzer_dispatched_bucket" {
  type        = string
  description = "The s3 bucket which the analyzer stores items that have been processed"
}

variable "analyzer_dispatcher_queue" {
  type        = string
  description = "Main queue for the dispatcher"
}

variable "analyzer_dispatcher_dead_letter_queue" {
  type        = string
  description = "Dead letter queue for the analyzer services"
}

variable "analyzer_matched_subgraphs_bucket" {
  type        = string
  description = "The s3 bucket used for storing matches"
}

variable "analyzer_executor_queue" {
  type        = string
  description = "Main queue for the executor"
}

variable "dgraph_replicas" {
  type    = number
  default = 1
}

variable "dgraph_shards" {
  type    = number
  default = 1
}

variable "engagement_creator_queue" {
  type = string
}

variable "redis_endpoint" {
  type        = string
  description = "Where can services find Redis?"
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

variable "plugin_registry_db_hostname" {
  type        = string
  description = "What is the host for the plugin registry table?"
}

variable "plugin_registry_db_port" {
  type        = string
  description = "What is the port for the plugin registry table?"
}

variable "plugin_registry_db_username" {
  type        = string
  description = "What is the username for the plugin registry table?"
}

variable "plugin_registry_db_password" {
  type        = string
  description = "What is the password for the plugin registry table?"
}

variable "plugin_registry_kernel_artifact_url" {
  type        = string
  description = "URL specifying the kernel.tar.gz for the Firecracker VM"
}

variable "plugin_registry_rootfs_artifact_url" {
  type        = string
  description = "URL specifying the rootfs.tar.gz for the Firecracker VM"
}

variable "organization_management_db_hostname" {
  type        = string
  description = "What is the host for the organization management database?"
}

variable "organization_management_db_port" {
  type        = string
  description = "What is the port for the organization management database?"
}

variable "organization_management_db_username" {
  type        = string
  description = "What is the username for the organization management database?"
}

variable "organization_management_db_password" {
  type        = string
  description = "What is the password for the organization management database?"
}

variable "plugin_work_queue_db_hostname" {
  type        = string
  description = "What is the host for the plugin work queue table?"
}

variable "plugin_work_queue_db_port" {
  type        = string
  description = "What is the port for the plugin work queue table?"
}

variable "plugin_work_queue_db_username" {
  type        = string
  description = "What is the username for the plugin work queue table?"
}

variable "plugin_work_queue_db_password" {
  type        = string
  description = "What is the password for the plugin work queue table?"
}

variable "plugin_s3_bucket_aws_account_id" {
  type        = string
  description = "The account id that owns the bucket where plugins are stored"
}

variable "plugin_s3_bucket_name" {
  type        = string
  description = "The name of the bucket where plugins are stored"
}

variable "num_graph_mergers" {
  type        = number
  default     = 1
  description = "The number of graph merger instances to run."
}

variable "graph_merger_queue" {
  type = string
}

variable "graph_merger_dead_letter_queue" {
  type = string
}

variable "test_user_name" {
  type        = string
  description = "The name of the test user"
}

variable "model_plugins_bucket" {
  type        = string
  description = "The s3 bucket used for storing plugins"
}

variable "num_node_identifiers" {
  type        = number
  default     = 1
  description = "The number of node identifiers to run."
}

variable "num_node_identifier_retries" {
  type        = number
  default     = 1
  description = "The number of node identifier retries to run."
}

variable "node_identifier_queue" {
  type = string
}

variable "node_identifier_dead_letter_queue" {
  type = string
}

variable "node_identifier_retry_queue" {
  type = string
}

variable "unid_subgraphs_generated_bucket" {
  type        = string
  description = "The destination bucket for unidentified subgraphs. Used by generators."
}

variable "subgraphs_merged_bucket" {
  type        = string
  description = "The destination bucket for merged subgraphs. Used by Graph Merger."
}

variable "subgraphs_generated_bucket" {
  type        = string
  description = "The destination bucket for generated subgraphs. Used by Node identifier."
}

variable "user_auth_table" {
  type        = string
  description = "What is the name of the DynamoDB user auth table?"
}

variable "user_session_table" {
  type        = string
  description = "What is the name of the DynamoDB user session table?"
}

variable "sysmon_generator_queue" {
  type = string
}

variable "sysmon_generator_dead_letter_queue" {
  type = string
}

variable "osquery_generator_queue" {
  type = string
}

variable "osquery_generator_dead_letter_queue" {
  type = string
}

variable "tracing_endpoint" {
  type = string
  # if nothing is passed in we default to "${attr.unique.network.ip-address}" in locals.
  # Using a variable isn't allowed here though :(
  default = ""
}

locals {
  dgraph_zero_grpc_private_port_base  = 5080
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
  }]

  # String that contains all of the Zeros for the Alphas to talk to and ensure they don't go down when one dies
  zero_alpha_connect_str = join(",", [for zero_id in range(0, var.dgraph_replicas) : "localhost:${local.dgraph_zero_grpc_private_port_base + zero_id}"])

  # String that contains all of the running Alphas for clients connecting to Dgraph (so they can do loadbalancing)
  alpha_grpc_connect_str = join(",", [for alpha in local.dgraph_alphas : "localhost:${alpha.grpc_public_port}"])

  _redis_trimmed = trimprefix(var.redis_endpoint, "redis://")
  _redis         = split(":", local._redis_trimmed)
  redis_host     = local._redis[0]
  redis_port     = local._redis[1]

  # Tracing endpoints
  # We currently use both the zipkin v2 endpoint for consul, python and typescript instrumentation and the jaeger udp
  # agent endpoint for rust instrumentation. These will be consolidated in the future
  tracing_endpoint             = (var.tracing_endpoint == "") ? attr.unique.network.ip-address : var.tracing_endpoint
  tracing_jaeger_endpoint_host = local.tracing_endpoint
  tracing_jaeger_endpoint_port = 6831
  tracing_zipkin_endpoint      = "http://${local.tracing_endpoint}:9411/api/v2/spans"

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
          type = "volume"
          target = "/dgraph"
          source = "grapl-dgraph"
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
            type = "volume"
            target = "/dgraph"
            source = "grapl-dgraph"
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
                  local_path_port = 6080
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
            type = "volume"
            target = "/dgraph"
            source = "grapl-dgraph"
            readonly = false
          }

          ports = ["dgraph-alpha-port"]
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
                  local_path_port = 8080
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

  group "graph-merger" {
    count = var.num_graph_mergers

    network {
      mode = "bridge"
    }

    task "graph-merger" {
      driver = "docker"

      config {
        image = var.container_images["graph-merger"]
      }

      # This writes an env files that gets read by nomad automatically
      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      env {
        AWS_REGION         = var.aws_region
        RUST_LOG           = var.rust_log
        RUST_BACKTRACE     = local.rust_backtrace
        REDIS_ENDPOINT     = var.redis_endpoint
        MG_ALPHAS          = local.alpha_grpc_connect_str
        GRAPL_SCHEMA_TABLE = var.schema_table_name
        # https://github.com/grapl-security/grapl/blob/18b229e824fae99fa2d600750dd3b17387611ef4/pulumi/grapl/__main__.py#L165
        DEST_BUCKET_NAME                = var.subgraphs_merged_bucket
        SOURCE_QUEUE_URL                = var.graph_merger_queue
        DEAD_LETTER_QUEUE_URL           = var.graph_merger_dead_letter_queue
        OTEL_EXPORTER_JAEGER_AGENT_HOST = local.tracing_jaeger_endpoint_host
        OTEL_EXPORTER_JAEGER_AGENT_PORT = local.tracing_jaeger_endpoint_port
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
    count = var.num_node_identifiers

    network {
      mode = "bridge"
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

      env {
        AWS_REGION                  = var.aws_region
        RUST_LOG                    = var.rust_log
        RUST_BACKTRACE              = local.rust_backtrace
        REDIS_ENDPOINT              = var.redis_endpoint
        MG_ALPHAS                   = local.alpha_grpc_connect_str # alpha_grpc_connect_str won't work if network mode = grapl network
        GRAPL_SCHEMA_TABLE          = var.schema_table_name
        GRAPL_DYNAMIC_SESSION_TABLE = var.session_table_name
        # https://github.com/grapl-security/grapl/blob/18b229e824fae99fa2d600750dd3b17387611ef4/pulumi/grapl/__main__.py#L156
        DEST_BUCKET_NAME                = var.subgraphs_generated_bucket
        SOURCE_QUEUE_URL                = var.node_identifier_queue
        DEAD_LETTER_QUEUE_URL           = var.node_identifier_retry_queue
        OTEL_EXPORTER_JAEGER_AGENT_HOST = local.tracing_jaeger_endpoint_host
        OTEL_EXPORTER_JAEGER_AGENT_PORT = local.tracing_jaeger_endpoint_port
      }

      service {
        name = "node-identifier"
      }
    }
  }

  group "node-identifier-retry" {
    count = var.num_node_identifier_retries

    network {
      mode = "bridge"
    }

    task "node-identifier-retry" {
      driver = "docker"

      config {
        image = var.container_images["node-identifier-retry"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      env {
        AWS_REGION                      = var.aws_region
        RUST_LOG                        = var.rust_log
        RUST_BACKTRACE                  = local.rust_backtrace
        REDIS_ENDPOINT                  = var.redis_endpoint
        MG_ALPHAS                       = local.alpha_grpc_connect_str
        GRAPL_SCHEMA_TABLE              = var.schema_table_name
        GRAPL_DYNAMIC_SESSION_TABLE     = var.session_table_name
        DEST_BUCKET_NAME                = var.subgraphs_generated_bucket
        SOURCE_QUEUE_URL                = var.node_identifier_retry_queue
        DEAD_LETTER_QUEUE_URL           = var.node_identifier_dead_letter_queue
        OTEL_EXPORTER_JAEGER_AGENT_HOST = local.tracing_jaeger_endpoint_host
        OTEL_EXPORTER_JAEGER_AGENT_PORT = local.tracing_jaeger_endpoint_port
      }

      service {
        name = "node-identifier-retry"
      }

    }
  }

  group "analyzer-dispatcher" {

    task "analyzer-dispatcher" {
      driver = "docker"

      config {
        image = var.container_images["analyzer-dispatcher"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      env {
        # AWS vars
        AWS_REGION = var.aws_region
        # rust vars
        RUST_LOG       = var.rust_log
        RUST_BACKTRACE = local.rust_backtrace
        # service vars
        GRAPL_ANALYZERS_BUCKET = var.analyzer_bucket
        DEST_BUCKET_NAME       = var.analyzer_dispatched_bucket
        SOURCE_QUEUE_URL       = var.analyzer_dispatcher_queue
        DEAD_LETTER_QUEUE_URL  = var.analyzer_dispatcher_dead_letter_queue
        # tracing vars
        OTEL_EXPORTER_JAEGER_AGENT_HOST = local.tracing_jaeger_endpoint_host
        OTEL_EXPORTER_JAEGER_AGENT_PORT = local.tracing_jaeger_endpoint_port
      }

      service {
        name = "analyzer-dispatcher"
      }

    }

  }

  group "analyzer-executor" {
    network {
      mode = "bridge"
    }

    task "analyzer-executor" {
      driver = "docker"

      config {
        image = var.container_images["analyzer-executor"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      env {
        # AWS vars
        AWS_DEFAULT_REGION = var.aws_region
        # python vars
        GRAPL_LOG_LEVEL = var.py_log_level
        # dgraph vars
        MG_ALPHAS = local.alpha_grpc_connect_str
        # service vars
        GRAPL_ANALYZER_MATCHED_SUBGRAPHS_BUCKET = var.analyzer_matched_subgraphs_bucket
        GRAPL_ANALYZERS_BUCKET                  = var.analyzer_bucket
        GRAPL_MODEL_PLUGINS_BUCKET              = var.model_plugins_bucket
        SOURCE_QUEUE_URL                        = var.analyzer_executor_queue
        GRPC_ENABLE_FORK_SUPPORT                = 1
        HITCACHE_ADDR                           = local.redis_host
        HITCACHE_PORT                           = local.redis_port
        IS_RETRY                                = "False"
        MESSAGECACHE_ADDR                       = local.redis_host
        MESSAGECACHE_PORT                       = local.redis_port
        OTEL_EXPORTER_ZIPKIN_ENDPOINT           = local.tracing_zipkin_endpoint
      }
    }

    service {
      name = "analyzer-executor"
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

  group "engagement-creator" {
    network {
      mode = "bridge"
    }

    task "engagement-creator" {
      driver = "docker"

      config {
        image = var.container_images["engagement-creator"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      env {
        # AWS vars
        AWS_DEFAULT_REGION = var.aws_region
        # python vars
        GRAPL_LOG_LEVEL = var.py_log_level
        # dgraph vars
        MG_ALPHAS = local.alpha_grpc_connect_str

        # service vars
        SOURCE_QUEUE_URL = var.engagement_creator_queue
        # Tracing endpoint
        OTEL_EXPORTER_ZIPKIN_ENDPOINT = local.tracing_zipkin_endpoint
      }
    }

    service {
      name = "engagement-creator"
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

  group "graphql-endpoint" {
    network {
      mode = "bridge"
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
        OTEL_EXPORTER_ZIPKIN_ENDPOINT = local.tracing_zipkin_endpoint
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

  group "model-plugin-deployer" {
    network {
      mode = "bridge"
      port "model-plugin-deployer" {
      }
    }

    task "model-plugin-deployer" {
      driver = "docker"

      config {
        image = var.container_images["model-plugin-deployer"]
        ports = ["model-plugin-deployer"]
      }

      env {
        RUST_LOG                         = var.rust_log
        RUST_BACKTRACE                   = local.rust_backtrace
        GRAPL_MODEL_PLUGIN_DEPLOYER_PORT = "${NOMAD_PORT_model-plugin-deployer}"
      }
    }

    service {
      name = "model-plugin-deployer"
      port = "model-plugin-deployer"
      connect {
        sidecar_service {}
      }
    }
  }


  group "web-ui" {
    network {
      mode = "bridge"

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

      env {
        # For the DynamoDB client
        AWS_REGION = var.aws_region

        GRAPL_USER_AUTH_TABLE    = var.user_auth_table
        GRAPL_USER_SESSION_TABLE = var.user_session_table

        GRAPL_WEB_UI_BIND_ADDRESS            = "0.0.0.0:${NOMAD_PORT_web-ui-port}"
        GRAPL_GRAPHQL_ENDPOINT               = "http://${NOMAD_UPSTREAM_ADDR_graphql-endpoint}"
        GRAPL_MODEL_PLUGIN_DEPLOYER_ENDPOINT = "http://TODO:1111" # Note - MPD is being replaced by a Rust service.
        RUST_LOG                             = var.rust_log
        RUST_BACKTRACE                       = local.rust_backtrace
        OTEL_EXPORTER_JAEGER_AGENT_HOST      = local.tracing_jaeger_endpoint_host
        OTEL_EXPORTER_JAEGER_AGENT_PORT      = local.tracing_jaeger_endpoint_port
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
    }
  }

  group "sysmon-generator" {
    network {
      mode = "bridge"
    }

    task "sysmon-generator" {
      driver = "docker"

      config {
        image = var.container_images["sysmon-generator"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      env {
        DEST_BUCKET_NAME                = var.unid_subgraphs_generated_bucket
        DEAD_LETTER_QUEUE_URL           = var.sysmon_generator_dead_letter_queue
        SOURCE_QUEUE_URL                = var.sysmon_generator_queue
        AWS_REGION                      = var.aws_region
        REDIS_ENDPOINT                  = var.redis_endpoint
        RUST_LOG                        = var.rust_log
        RUST_BACKTRACE                  = local.rust_backtrace
        OTEL_EXPORTER_JAEGER_AGENT_HOST = local.tracing_jaeger_endpoint_host
        OTEL_EXPORTER_JAEGER_AGENT_PORT = local.tracing_jaeger_endpoint_port
      }
    }
  }

  group "osquery-generator" {
    network {
      mode = "bridge"
    }

    task "osquery-generator" {
      driver = "docker"

      config {
        image = var.container_images["osquery-generator"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      env {
        DEST_BUCKET_NAME                = var.unid_subgraphs_generated_bucket
        DEAD_LETTER_QUEUE_URL           = var.osquery_generator_dead_letter_queue
        SOURCE_QUEUE_URL                = var.osquery_generator_queue
        AWS_REGION                      = var.aws_region
        REDIS_ENDPOINT                  = var.redis_endpoint
        RUST_LOG                        = var.rust_log
        RUST_BACKTRACE                  = local.rust_backtrace
        OTEL_EXPORTER_JAEGER_AGENT_HOST = local.tracing_jaeger_endpoint_host
        OTEL_EXPORTER_JAEGER_AGENT_PORT = local.tracing_jaeger_endpoint_port
      }
    }
  }

  group "organization-management" {
    network {
      mode = "bridge"
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

      env {
        AWS_REGION                           = var.aws_region
        NOMAD_SERVICE_ADDRESS                = "${attr.unique.network.ip-address}:4646"
        ORGANIZATION_MANAGEMENT_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_organization-management-port}"
        RUST_BACKTRACE                       = local.rust_backtrace
        RUST_LOG                             = var.rust_log
        ORGANIZATION_MANAGEMENT_DB_HOSTNAME  = var.organization_management_db_hostname
        ORGANIZATION_MANAGEMENT_DB_PASSWORD  = var.organization_management_db_password
        ORGANIZATION_MANAGEMENT_DB_PORT      = var.organization_management_db_port
        ORGANIZATION_MANAGEMENT_DB_USERNAME  = var.organization_management_db_username
        OTEL_EXPORTER_JAEGER_AGENT_HOST      = local.tracing_jaeger_endpoint_host
        OTEL_EXPORTER_JAEGER_AGENT_PORT      = local.tracing_jaeger_endpoint_port
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

  group "plugin-registry" {
    network {
      mode = "bridge"

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

      env {
        AWS_REGION                          = var.aws_region
        NOMAD_SERVICE_ADDRESS               = "${attr.unique.network.ip-address}:4646"
        PLUGIN_REGISTRY_BIND_ADDRESS        = "0.0.0.0:${NOMAD_PORT_plugin-registry-port}"
        PLUGIN_REGISTRY_DB_HOSTNAME         = var.plugin_registry_db_hostname
        PLUGIN_REGISTRY_DB_PASSWORD         = var.plugin_registry_db_password
        PLUGIN_REGISTRY_DB_PORT             = var.plugin_registry_db_port
        PLUGIN_REGISTRY_DB_USERNAME         = var.plugin_registry_db_username
        PLUGIN_BOOTSTRAP_CONTAINER_IMAGE    = var.container_images["plugin-bootstrap"]
        PLUGIN_REGISTRY_KERNEL_ARTIFACT_URL = var.plugin_registry_kernel_artifact_url
        PLUGIN_REGISTRY_ROOTFS_ARTIFACT_URL = var.plugin_registry_rootfs_artifact_url
        # Plugin Execution code/image doesn't exist yet; change this once it does!
        PLUGIN_EXECUTION_CONTAINER_IMAGE = "grapl/plugin-execution-sidecar-TODO"
        PLUGIN_S3_BUCKET_AWS_ACCOUNT_ID  = var.plugin_s3_bucket_aws_account_id
        PLUGIN_S3_BUCKET_NAME            = var.plugin_s3_bucket_name
        RUST_BACKTRACE                   = local.rust_backtrace
        RUST_LOG                         = var.rust_log
        OTEL_EXPORTER_JAEGER_AGENT_HOST  = local.tracing_jaeger_endpoint_host
        OTEL_EXPORTER_JAEGER_AGENT_PORT  = local.tracing_jaeger_endpoint_port
      }
    }

    service {
      name = "plugin-registry"
      port = "plugin-registry-port"
      connect {
        sidecar_service {
        }
      }
    }
  }

  group "plugin-work-queue" {
    network {
      mode = "bridge"

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

      env {
        PLUGIN_WORK_QUEUE_BIND_ADDRESS  = "0.0.0.0:${NOMAD_PORT_plugin-work-queue-port}"
        PLUGIN_WORK_QUEUE_DB_HOSTNAME   = var.plugin_work_queue_db_hostname
        PLUGIN_WORK_QUEUE_DB_PASSWORD   = var.plugin_work_queue_db_password
        PLUGIN_WORK_QUEUE_DB_PORT       = var.plugin_work_queue_db_port
        PLUGIN_WORK_QUEUE_DB_USERNAME   = var.plugin_work_queue_db_username
        PLUGIN_S3_BUCKET_AWS_ACCOUNT_ID = var.plugin_s3_bucket_aws_account_id
        PLUGIN_S3_BUCKET_NAME           = var.plugin_s3_bucket_name
        RUST_BACKTRACE                  = local.rust_backtrace
        RUST_LOG                        = var.rust_log
        OTEL_EXPORTER_JAEGER_AGENT_HOST = local.tracing_jaeger_endpoint_host
        OTEL_EXPORTER_JAEGER_AGENT_PORT = local.tracing_jaeger_endpoint_port
      }
    }

    service {
      name = "plugin-work-queue"
      port = "plugin-work-queue-port"
      connect {
        sidecar_service {
        }
      }
    }
  }
}
