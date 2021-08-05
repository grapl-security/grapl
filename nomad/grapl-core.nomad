variable "rust_log" {
  type        = string
  default     = "INFO"
  description = "Controls the logging behavior of Rust-based services."
}

variable "container_registry" {
  type        = string
  default     = "localhost:5000"
  description = "The container registry in which we can find Grapl services."
}

variable "aws_region" {
  type    = string
  default = "us-west-2"
}

variable "aws_sqs_url" {
  type = string
}

variable "aws_account_id" {
  type    = string
  default = "000000000000"
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

# https://github.com/grapl-security/grapl/blob/af6f2c197d52e9941047aab813c30d2cbfd54523/pulumi/infra/dynamodb.py#L118
variable "session_table" {
  type    = string
  default = "dynamic_session_table"
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

    network {
      mode = "bridge"
    }

    task "graph-merger" {
      driver = "docker"

      config {
        image = "${var.container_registry}/grapl/graph-merger:${var.graph_merger_tag}"
      }

      env {
        RUST_LOG           = "${var.rust_log}"
        REDIS_ENDPOINT     = "${var.redis_endpoint}"
        MG_ALPHAS          = "${local.alpha_grpc_connect_str}"
        GRAPL_SCHEMA_TABLE = "${var.schema_table_name}"
        AWS_REGION         = "${var.aws_region}"
        # https://github.com/grapl-security/grapl/blob/18b229e824fae99fa2d600750dd3b17387611ef4/pulumi/grapl/__main__.py#L165
        DEST_BUCKET_NAME      = "subgraphs-merged-bucket"
        SOURCE_QUEUE_URL      = "${var.aws_sqs_url}/${var.aws_account_id}/graph-merger-queue"
        DEAD_LETTER_QUEUE_URL = "${var.aws_sqs_url}/${var.aws_account_id}/graph-merger-dead-letter-queue"
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
        image = "${var.container_registry}/grapl/node-identifier:${var.node_identifier_tag}"
      }

      env {
        RUST_LOG                    = "${var.rust_log}"
        REDIS_ENDPOINT              = "${var.redis_endpoint}"
        MG_ALPHAS                   = "${local.alpha_grpc_connect_str}"
        GRAPL_SCHEMA_TABLE          = "${var.schema_table_name}"
        AWS_REGION                  = "${var.aws_region}"
        GRAPL_DYNAMIC_SESSION_TABLE = "${var.session_table}"
        # https://github.com/grapl-security/grapl/blob/18b229e824fae99fa2d600750dd3b17387611ef4/pulumi/grapl/__main__.py#L156
        DEST_BUCKET_NAME      = "subgraphs-generated-bucket"
        SOURCE_QUEUE_URL      = "${var.aws_sqs_url}/${var.aws_account_id}/node-identifier-queue"
        DEAD_LETTER_QUEUE_URL = "${var.aws_sqs_url}/${var.aws_account_id}/node-identifier-dead-letter-queue"
      }
    }
  }

  group "grapl-node-identifier-retry" {
    count = var.num_node_identifier_retries

    network {
      mode = "bridge"
    }

    task "node-identifier-retry" {
      driver = "docker"

      config {
        image = "${var.container_registry}/grapl/node-identifier-retry:${var.node_identifier_tag}"
      }

      env {
        RUST_LOG                    = "${var.rust_log}"
        REDIS_ENDPOINT              = "${var.redis_endpoint}"
        MG_ALPHAS                   = "${local.alpha_grpc_connect_str}"
        GRAPL_SCHEMA_TABLE          = "${var.schema_table_name}"
        AWS_REGION                  = "${var.aws_region}"
        GRAPL_DYNAMIC_SESSION_TABLE = "${var.session_table}"
        DEST_BUCKET_NAME            = "subgraphs-generated-bucket"
        SOURCE_QUEUE_URL            = "${var.aws_sqs_url}/${var.aws_account_id}/node-identifier-retry-queue"
        DEAD_LETTER_QUEUE_URL       = "${var.aws_sqs_url}/${var.aws_account_id}/node-identifier-retry-dead-letter-queue"
      }
    }
  }
}