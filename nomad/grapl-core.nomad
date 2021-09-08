variable "rust_log" {
  type        = string
  description = "Controls the logging behavior of Rust-based services."
}

variable "container_registry" {
  type        = string
  default     = "localhost:5000"
  description = "The container registry in which we can find Grapl services."
}

variable "aws_access_key_id" {
  type        = string
  default     = "test"
  description = "The aws access key id used to interact with AWS."
}

variable "aws_access_key_secret" {
  type        = string
  default     = "test"
  description = "The aws access key secret used to interact with AWS."
}

variable "aws_endpoint" {
  type        = string
  description = <<EOF
  The endpoint in which we can expect to find and interact with AWS. 
  It accepts a special sentinel value, USE_LOCALSTACK_SENTINEL_VALUE, if the
  user wishes to contact Localstack.

  Prefer using `local.aws_endpoint`.
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

variable "analyzer_dispatcher_tag" {
  type        = string
  default     = "latest"
  description = "The tagged version of the analyzer-dispatcher we should deploy."
}

variable "analyzer_matched_subgraphs_bucket" {
  type        = string
  description = "The s3 bucket used for storing matches"
}

variable "analyzer_executor_queue" {
  type        = string
  description = "Main queue for the executor"
}

variable "analyzer_executor_tag" {
  type        = string
  default     = "latest"
  description = "The tagged version of the analyzer-executor we should deploy."
}

variable "dgraph_tag" {
  type        = string
  default     = "v21.03.1"
  description = "The tag we should use when pulling dgraph."
}

variable "dgraph_replicas" {
  type    = number
  default = 1
}

variable "dgraph_shards" {
  type    = number
  default = 1
}

variable "redis_endpoint" {
  type        = string
  description = "Where can services find redis?"
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

variable "num_graph_mergers" {
  type        = number
  default     = 1
  description = "The number of graph merger instances to run."
}

variable "graph_merger_tag" {
  type        = string
  default     = "latest"
  description = "The tagged version of the graph_merger we should deploy."
}

variable "graph_merger_queue" {
  type = string
}

variable "graph_merger_dead_letter_queue" {
  type = string
}

variable "grapl_test_user_name" {
  type        = string
  description = "The name of the test user"
  default     = "grapl-test-user"
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

variable "node_identifier_tag" {
  type        = string
  default     = "latest"
  description = "The tagged version of the node_identifier and the node_identifier_retry we should deploy."
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

variable "provisioner_tag" {
  type        = string
  default     = "latest"
  description = "The tagged version of the provisioner we should deploy."
}

variable "subgraphs_merged_bucket" {
  type        = string
  description = "The destination bucket for merged subgraphs. Used by Graph Merger."
}

variable "subgraphs_generated_bucket" {
  type        = string
  description = "The destination bucket for generated subgraphs. Used by Node identifier."
}

variable "engagement_view_tag" {
  type        = string
  default     = "latest"
  description = "The image tag for the engagement view."
}

variable "ux_bucket" {
  type        = string
  description = "The grapl UX bucket for the engagement view."
}

variable "graphql_endpoint_tag" {
  type        = string
  default     = "latest"
  description = "The image tag for the graphql endpoint docker image."
}

variable "deployment_name" {
  type        = string
  description = "The deployment name"
}

variable "user_auth_table" {
  type        = string
  description = "What is the name of the user auth table?"
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

  # Used for local development
  local_aws_endpoint = "http://${attr.unique.network.ip-address}:4566"

  # AWS endpoint to use when interacting with AWS. Prefer this over var.aws_endpoint
  aws_endpoint = var.aws_endpoint != "USE_LOCALSTACK_SENTINEL_VALUE" ? var.aws_endpoint : local.local_aws_endpoint

  local_redis_endpoint = "redis://${attr.unique.network.ip-address}:6379"
  # redis endpoint to use when interacting with redis. Prefer this over var.redis_endpoint
  redis_endpoint = var.redis_endpoint != "USE_REDIS_SENTINEL_VALUE" ? var.redis_endpoint : local.local_redis_endpoint

  redis_trimmed = trimprefix(local.redis_endpoint, "redis://")
  redis         = split(":", local.redis_trimmed)
  redis_host    = local.redis[0]
  redis_port    = local.redis[1]
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
        image = "dgraph/dgraph:${var.dgraph_tag}"
        args = [
          "dgraph",
          "zero",
          "--my", "localhost:${local.dgraph_zero_grpc_private_port_base}",
          "--replicas", "${var.dgraph_replicas}",
          "--raft", "idx=1",
        ]
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
      }

      task "dgraph-zero" {
        driver = "docker"

        config {
          image = "dgraph/dgraph:${var.dgraph_tag}"
          args = [
            "dgraph",
            "zero",
            "--my", "localhost:${zero.value.grpc_private_port}",
            "--replicas", "${var.dgraph_replicas}",
            "--raft", "idx=${zero.value.id + 1}",
            "--port_offset", "${zero.value.id}",
            "--peer", "localhost:${local.dgraph_zero_grpc_private_port_base}"
          ]
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
            }
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
      }

      task "dgraph-alpha" {
        driver = "docker"

        config {
          image = "dgraph/dgraph:${var.dgraph_tag}"
          args = [
            "dgraph",
            "alpha",
            "--my", "localhost:${alpha.value.grpc_private_port}",
            "--port_offset", "${alpha.value.id}",
            "--zero", "${local.zero_alpha_connect_str}"
          ]
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
          sidecar_service {}
        }
      }
    }
  }

  group "grapl-graph-merger" {
    count = var.num_graph_mergers

    //    network {
    //      mode = "bridge"
    //    }

    task "graph-merger" {
      driver = "docker"

      config {
        image        = "${var.container_registry}/grapl/graph-merger:${var.graph_merger_tag}"
        network_mode = "grapl-network"
      }

      env {
        GRAPL_AWS_ENDPOINT          = local.aws_endpoint
        GRAPL_AWS_ACCESS_KEY_ID     = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET = var.aws_access_key_secret
        AWS_DEFAULT_REGION          = var.aws_region # boto3 prefers this one
        AWS_REGION                  = var.aws_region
        RUST_LOG                    = var.rust_log
        RUST_BACKTRACE              = 1
        REDIS_ENDPOINT              = local.redis_endpoint
        MG_ALPHAS                   = local.alpha_grpc_connect_str
        GRAPL_SCHEMA_TABLE          = var.schema_table_name
        # https://github.com/grapl-security/grapl/blob/18b229e824fae99fa2d600750dd3b17387611ef4/pulumi/grapl/__main__.py#L165
        DEST_BUCKET_NAME      = var.subgraphs_merged_bucket
        SOURCE_QUEUE_URL      = var.graph_merger_queue
        DEAD_LETTER_QUEUE_URL = var.graph_merger_dead_letter_queue
      }
    }

    //    service {
    //      connect {
    //        sidecar_service {
    //          proxy {
    //            dynamic "upstreams" {
    //              iterator = alpha
    //              for_each = local.dgraph_alphas
    //
    //              content {
    //                destination_name = "dgraph-alpha-${alpha.value.id}-grpc-public"
    //                local_bind_port  = alpha.value.grpc_public_port
    //              }
    //            }
    //          }
    //        }
    //      }
    //    }
  }

  group "provisioner" {
    network {
      mode = "bridge"
    }

    task "provisioner" {
      driver = "docker"

      config {
        image = "${var.container_registry}/grapl/provisioner:${var.provisioner_tag}"
      }

      lifecycle {
        hook = "poststart"
        # Ephemeral, not long-lived
        sidecar = false
      }

      env {
        MG_ALPHAS                     = local.alpha_grpc_connect_str
        DEPLOYMENT_NAME               = var.deployment_name
        GRAPL_AWS_ENDPOINT            = local.aws_endpoint
        GRAPL_AWS_ACCESS_KEY_ID       = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET   = var.aws_access_key_secret
        AWS_DEFAULT_REGION            = var.aws_region # boto3 prefers this one
        AWS_REGION                    = var.aws_region
        GRAPL_SCHEMA_TABLE            = var.schema_table_name
        GRAPL_SCHEMA_PROPERTIES_TABLE = var.schema_properties_table_name
        GRAPL_USER_AUTH_TABLE         = var.user_auth_table
        GRAPL_TEST_USER_NAME          = var.grapl_test_user_name
        GRAPL_LOG_LEVEL               = var.rust_log # TODO: revisit
      }
    }

    service {
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

  group "grapl-node-identifier" {
    count = var.num_node_identifiers

    network {
      mode = "bridge"
    }

    task "node-identifier" {
      driver = "docker"

      config {
        image        = "${var.container_registry}/grapl/node-identifier:${var.node_identifier_tag}"
        network_mode = "grapl-network"
      }

      env {
        GRAPL_AWS_ENDPOINT          = local.aws_endpoint
        GRAPL_AWS_ACCESS_KEY_ID     = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET = var.aws_access_key_secret
        AWS_DEFAULT_REGION          = var.aws_region # boto3 prefers this one
        AWS_REGION                  = var.aws_region
        RUST_LOG                    = var.rust_log
        RUST_BACKTRACE              = 1
        REDIS_ENDPOINT              = local.redis_endpoint
        MG_ALPHAS                   = local.alpha_grpc_connect_str # alpha_grpc_connect_str won't work if network mode = grapl network
        GRAPL_SCHEMA_TABLE          = var.schema_table_name
        GRAPL_DYNAMIC_SESSION_TABLE = var.session_table_name
        # https://github.com/grapl-security/grapl/blob/18b229e824fae99fa2d600750dd3b17387611ef4/pulumi/grapl/__main__.py#L156
        DEST_BUCKET_NAME      = var.subgraphs_generated_bucket
        SOURCE_QUEUE_URL      = var.node_identifier_queue
        DEAD_LETTER_QUEUE_URL = var.node_identifier_retry_queue
      }
    }
  }

  group "grapl-node-identifier-retry" {
    count = var.num_node_identifier_retries

    //    network {
    //      mode = "bridge"
    //    }

    task "node-identifier-retry" {
      driver = "docker"

      config {
        image        = "${var.container_registry}/grapl/node-identifier-retry:${var.node_identifier_tag}"
        network_mode = "grapl-network"
      }

      env {
        GRAPL_AWS_ENDPOINT          = local.aws_endpoint
        GRAPL_AWS_ACCESS_KEY_ID     = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET = var.aws_access_key_secret
        AWS_DEFAULT_REGION          = var.aws_region # boto3 prefers this one
        AWS_REGION                  = var.aws_region
        RUST_LOG                    = var.rust_log
        RUST_BACKTRACE              = 1
        REDIS_ENDPOINT              = local.redis_endpoint
        MG_ALPHAS                   = local.alpha_grpc_connect_str
        GRAPL_SCHEMA_TABLE          = var.schema_table_name
        GRAPL_DYNAMIC_SESSION_TABLE = var.session_table_name
        DEST_BUCKET_NAME            = var.subgraphs_generated_bucket
        SOURCE_QUEUE_URL            = var.node_identifier_retry_queue
        DEAD_LETTER_QUEUE_URL       = var.node_identifier_dead_letter_queue
      }
    }
  }

  group "analyzer-dispatcher" {

    task "analyzer-dispatcher" {
      driver = "docker"

      config {
        image        = "${var.container_registry}/grapl/analyzer-dispatcher:${var.analyzer_dispatcher_tag}"
        network_mode = "grapl-network"
      }

      env {
        # AWS vars
        AWS_REGION                  = var.aws_region
        GRAPL_AWS_ACCESS_KEY_ID     = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET = var.aws_access_key_secret
        GRAPL_AWS_ENDPOINT          = var.aws_endpoint
        # rust vars
        RUST_LOG       = var.rust_log
        RUST_BACKTRACE = 1
        # service vars
        GRAPL_ANALYZERS_BUCKET = var.analyzer_bucket
        DEST_BUCKET_NAME       = var.analyzer_dispatched_bucket
        DEAD_LETTER_QUEUE_URL  = var.analyzer_dispatcher_queue
        SOURCE_QUEUE_URL       = var.analyzer_dispatcher_dead_letter_queue
      }
    }

  }

  group "analyzer-executor" {
    task "analyzer-executor" {
      driver = "docker"

      config {
        image        = "${var.container_registry}/grapl/analyzer-executor:${var.analyzer_executor_tag}"
        network_mode = "grapl-network"
      }

      env {
        # AWS vars
        AWS_REGION                  = var.aws_region
        GRAPL_AWS_ACCESS_KEY_ID     = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET = var.aws_access_key_secret
        GRAPL_AWS_ENDPOINT          = local.aws_endpoint
        # python vars
        GRAPL_LOG_LEVEL = "INFO"
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
      }
    }
  }

  group "graphql-endpoint" {
    network {
      //      mode = "bridge"

      port "graphql-endpoint" {
        to = 5000
      }
    }

    // engagement-view just uploads the ux tarball. For the moment this is set to run as an init container but should
    // probably be run from Buildkite instead
    task "engagement-view" {
      driver = "docker"

      config {
        image        = "${var.container_registry}/grapl/engagement-view:${var.engagement_view_tag}"
        network_mode = "grapl-network"
      }

      lifecycle {
        hook    = "prestart"
        sidecar = false
      }

      env {
        RUST_LOG                    = var.rust_log
        GRAPL_UX_BUCKET             = var.ux_bucket
        AWS_REGION                  = var.aws_region
        GRAPL_AWS_ACCESS_KEY_ID     = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET = var.aws_access_key_secret
        GRAPL_AWS_ENDPOINT          = local.aws_endpoint
      }
    }

    task "graphql-endpoint" {
      driver = "docker"

      config {
        image        = "${var.container_registry}/grapl/graphql-endpoint:${var.graphql_endpoint_tag}"
        network_mode = "grapl-network"
      }

      env {
        DEPLOYMENT_NAME               = var.deployment_name
        RUST_LOG                      = var.rust_log
        GRAPL_UX_BUCKET               = var.ux_bucket
        AWS_REGION                    = var.aws_region
        MG_ALPHAS                     = local.alpha_grpc_connect_str
        GRAPL_SCHEMA_TABLE            = var.schema_table_name
        GRAPL_SCHEMA_PROPERTIES_TABLE = var.schema_properties_table_name
        GRAPL_AWS_ACCESS_KEY_ID       = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET   = var.aws_access_key_secret
        GRAPL_AWS_ENDPOINT            = local.aws_endpoint
        IS_LOCAL                      = "True"
        JWT_SECRET_ID                 = "JWT_SECRET_ID"
        PORT                          = 5000
      }
    }

    //    service {
    //      name = "graphql-endpoint"
    //      port = "graphql-endpoint"
    //
    //      connect {
    //        sidecar_service {}
    //      }
    //    }
  }
}