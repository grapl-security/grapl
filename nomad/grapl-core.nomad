variable "container_registry" {
    type = string
    default = "http://localhost:5000"
}

variable "dgraph_replicas" {
    type = number
    default = 1
}

variable "dgraph_shards" {
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

    zero_alpha_connect_str = join(",", [for zero_id in range(0, var.dgraph_replicas): "localhost:${zero_id + 5080}"])
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
        min_healthy_time = "1m"
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

                            # Connect this Zero follower to other Zero followers
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

                            # Connect this Alpha to Other Alphas
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
        }
    }
}