# For rust logging output
variable "RUST_LOG" {
    type = string
    default = "INFO"
}

# Which tag to use for grapl-specific images
variable "tag" {
    type = string
    default = "latest"
}

# The container registry that we can find grapl-specific services
variable "container_registry" {
    type = string
    default = "localhost:5000"
}

variable "aws_region" {
    type = string
    default = "us-west-2"
}

variable "aws_sqs_url" {
    type = string
}

variable "aws_account_id" {
    type = string
    default = "000000000000"
}

variable "dgraph_replicas" {
    type = number
    default = 1
}

variable "dgraph_shards" {
    type = number
    default = 1
}

# Where is redis located?
variable "redis_endpoint" {
    type = string
}

# What is the name of the schema table?
variable "schema_table_name" {
    type = string
}

variable "session_table" {
    type = string
    default = "dynamic_session_table"
}

variable "destination_bucket_name" {
    type = string
}

# How many graph mergers should be running in tandem
variable "graph_mergers" {
    type = number
    default = 1
}

# How many node identifiers should be running in tandem
variable "node_identifiers" {
    type = number
    default = 1
}

variable "node_identifiers_retry" {
    type = number
    default = 1
}

locals {
    # DGraph Alphas (shards * replicas)
    dgraph_alphas = [for alpha_id in range(0, var.dgraph_replicas * var.dgraph_shards): {
        id: alpha_id,
        grpc_private_port: alpha_id + 7080,
        grpc_public_port: alpha_id + 9080,
        http_port: alpha_id + 8080
    }]

    # DGraph Zeros (replicas)
    dgraph_zeros = [for zero_id in range(1, var.dgraph_replicas): {
        id: zero_id,
        grpc_private_port: zero_id + 5080,
    }]

    # String that contains all of the Zeros for the Alphas to talk to and ensure they don't go down when one dies
    zero_alpha_connect_str = join(",", [for zero_id in range(0, var.dgraph_replicas): "localhost:${zero_id + 5080}"])

    # String that contains all of the running Alphas for clients connecting to Dgraph (so they can do loadbalancing)
    alpha_grpc_connect_str = join(",", [for alpha in local.dgraph_alphas: "localhost:${alpha.grpc_public_port}"])
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
        canary = 1
        max_parallel = 1
        # The min amount of reported "healthy" time before a instance is considered healthy and an allocation is opened up for further updates
        min_healthy_time = "15s"
    }

    group "dgraph-zero-leader" {
        network {
            mode = "bridge"
        }

        task "dgraph-zero-leader" {
            driver = "docker"

            config {
                image = "dgraph/dgraph:latest"
                args = [
                    "dgraph",
                    "zero",
                    "--my=localhost:5080",
                    "--replicas", "${var.dgraph_replicas}",
                    "--raft", "idx=1",
                ]
            }
        }

        service {
            name = "dgraph-zero-leader-grpc-private"
            port = "5080"

            connect {
                sidecar_service {
                    proxy {
                        # Connect the Zero leader to the Zero followers
                        dynamic "upstreams" {
                            iterator = zero_follower
                            for_each = local.dgraph_zeros

                            content {
                                destination_name = "dgraph-zero-follower-${zero_follower.value.id}-grpc-private"
                                local_bind_port = zero_follower.value.grpc_private_port
                            }
                        }

                        # Connect this Zero leader to the Alphas
                        dynamic "upstreams" {
                            iterator = alpha
                            for_each = [for alpha in local.dgraph_alphas: alpha]

                            content {
                                destination_name = "dgraph-alpha-${alpha.value.id}-grpc-private"
                                local_bind_port = alpha.value.grpc_private_port
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
        labels = ["dgraph-zero-follower-${zero.value.id}"]

        content {
            network {
                mode = "bridge"
            }

            task "dgraph-zero-follower" {
                driver = "docker"

                config {
                    image = "dgraph/dgraph:latest"
                    args = [
                        "dgraph",
                        "zero",
                        "--my=localhost:${zero.value.grpc_private_port}",
                        "--replicas", "${var.dgraph_replicas}",
                        "--raft", "idx=${zero.value.id + 1}",
                        "-o", "${zero.value.id}",
                        "--peer", "localhost:5080"
                    ]
                }
            }
            service {
                name = "dgraph-zero-follower-${zero.value.id}-grpc-private"
                port = "${zero.value.grpc_private_port}"

                connect {
                    sidecar_service {
                        proxy {
                            # Connect to the Zero leader
                            upstreams {
                                destination_name = "dgraph-zero-leader-grpc-private"
                                local_bind_port = 5080
                            }

                            # Connect this Zero follower to other Zero followers (but not to itself, obviously)
                            dynamic "upstreams" {
                                iterator = zero_follower
                                for_each = [for zero_follower in local.dgraph_zeros: zero_follower if zero_follower.id != zero.value.id]

                                content {
                                    destination_name = "dgraph-zero-follower-${zero_follower.value.id}-grpc-private"
                                    local_bind_port = zero_follower.value.grpc_private_port
                                }
                            }

                            # Connect this Zero follower to the Alphas
                            dynamic "upstreams" {
                                iterator = alpha
                                for_each = [for alpha in local.dgraph_alphas: alpha]

                                content {
                                    destination_name = "dgraph-alpha-${alpha.value.id}-grpc-private"
                                    local_bind_port = alpha.value.grpc_private_port
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
        labels = ["dgraph-alpha-${alpha.value.id}"]

        content {
            network {
                mode = "bridge"
            }

            task "dgraph-alpha" {
                driver = "docker"

                config {
                    image = "dgraph/dgraph:latest"
                    args = [
                        "dgraph",
                        "alpha",
                        "--my=localhost:${alpha.value.grpc_private_port}",
                        "-o", "${alpha.value.id}",
                        "--zero", "${local.zero_alpha_connect_str}"
                    ]
                }
            }

            service {
                name = "dgraph-alpha-${alpha.value.id}-grpc-private"
                port = "${alpha.value.grpc_private_port}"

                connect {
                    sidecar_service {
                        proxy {
                            # Connect to the Zero leader
                            upstreams {
                                destination_name = "dgraph-zero-leader-grpc-private"
                                local_bind_port = 5080
                            }

                            # Connect this Alpha to Zero followers
                            dynamic "upstreams" {
                                iterator = zero_follower
                                for_each = [for zero_follower in local.dgraph_zeros: zero_follower]

                                content {
                                    destination_name = "dgraph-zero-follower-${zero_follower.value.id}-grpc-private"
                                    local_bind_port = zero_follower.value.grpc_private_port
                                }
                            }

                            # Connect this Alpha to Other Alphas (but not to itself, obviously)
                            dynamic "upstreams" {
                                iterator = alpha_peer
                                for_each = [for alpha_peer in local.dgraph_alphas: alpha_peer if alpha_peer.id != alpha.value.id]

                                content {
                                    destination_name = "dgraph-alpha-${alpha_peer.value.id}-grpc-private"
                                    local_bind_port = alpha_peer.value.grpc_private_port
                                }
                            }
                        }
                    }
                }
            }

            service {
                name = "dgraph-alpha-${alpha.value.id}-grpc-public"
                port = "${alpha.value.grpc_public_port}"

                connect {
                    sidecar_service { }
                }
            }

            service {
                name = "dgraph-alpha-${alpha.value.id}-http"
                port = "${alpha.value.http_port}"

                connect {
                    sidecar_service { }
                }
            }
        }
    }

    group "grapl-graph-merger" {
        count = var.graph_mergers

        network {
            mode = "bridge"
        }

        task "graph-merger" {
            driver = "docker"

            config {
                image = "${var.container_registry}/graph-merger:${var.tag}"
                args = ["/graph-merger"]
            }

            env {
                AWS_ACCESS_KEY_ID = "x"
                AWS_SECRET_ACCESS_KEY = "x"
                REDIS_ENDPOINT = "${var.redis_endpoint}"
                MG_ALPHAS = "${local.alpha_grpc_connect_str}"
                GRAPL_SCHEMA_TABLE = "${var.schema_table_name}"
                AWS_REGION = "${var.aws_region}"
                DEST_BUCKET_NAME = "${var.destination_bucket_name}"
                SOURCE_QUEUE_URL = "${var.aws_sqs_url}/${var.aws_account_id}/graph-merger-queue"
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
                                local_bind_port = alpha.value.grpc_public_port
                            }
                        }
                    }
                }
            }
        }
    }

    group "grapl-node-identifier" {
        count = var.node_identifiers

        network {
            mode = "bridge"
        }

        task "node-identifier" {
            driver = "docker"

            config {
                image = "${var.container_registry}/node-identifier:${var.tag}"
                args = ["/node-identifier"]
            }

            env {
                REDIS_ENDPOINT = "${var.redis_endpoint}"
                MG_ALPHAS = "${local.alpha_grpc_connect_str}"
                GRAPL_SCHEMA_TABLE = "${var.schema_table_name}"
                AWS_REGION = "${var.aws_region}"
                DYNAMIC_SESSION_TABLE = "${var.session_table}"
                DEST_BUCKET_NAME = "${var.destination_bucket_name}"
                SOURCE_QUEUE_URL = "${var.aws_sqs_url}/${var.aws_account_id}/node-identifier-queue"
                DEAD_LETTER_QUEUE_URL = "${var.aws_sqs_url}/${var.aws_account_id}/node-identifier-dead-letter-queue"
            }
        }
    }

    group "grapl-node-identifier-retry" {
        count = var.node_identifiers_retry

        network {
            mode = "bridge"
        }

        task "node-identifier-retry" {
            driver = "docker"

            config {
                image = "${var.container_registry}/node-identifier-retry:${var.tag}"
                args = ["/node-identifier-retry"]
            }

            env {
                REDIS_ENDPOINT = "${var.redis_endpoint}"
                MG_ALPHAS = "${local.alpha_grpc_connect_str}"
                GRAPL_SCHEMA_TABLE = "${var.schema_table_name}"
                AWS_REGION = "${var.aws_region}"
                DYNAMIC_SESSION_TABLE = "${var.session_table}"
                DEST_BUCKET_NAME = "${var.destination_bucket_name}"
                SOURCE_QUEUE_URL = "${var.aws_sqs_url}/${var.aws_account_id}/node-identifier-retry-queue"
                DEAD_LETTER_QUEUE_URL = "${var.aws_sqs_url}/${var.aws_account_id}/node-identifier-retry-dead-letter-queue"
            }
        }
    }
}