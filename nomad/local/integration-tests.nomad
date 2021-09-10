# This setup is inspired by the following forum discussion:
# https://discuss.hashicorp.com/t/best-practices-for-testing-against-services-in-nomad-consul-connect/29022
# We'll submit integration tests to Nomad as 
# 
variable "container_registry" {
  type        = string
  default     = ""
  description = "The container registry in which we can find Grapl services. Requires a trailing /"
}

variable "aws_region" {
  type = string
}

variable "deployment_name" {
  type        = string
  description = "The deployment name"
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

variable "_aws_endpoint" {
  type        = string
  description = "The endpoint in which we can expect to find and interact with AWS."
}

variable "_redis_endpoint" {
  type        = string
  description = "On which port can services find redis?"
}

variable "_kafka_endpoint" {
  type        = string
  description = "On which port can services find Kafka?"
}

locals {
  log_level = "DEBUG"

  # Prefer these over their `var` equivalents
  aws_endpoint   = replace(var._aws_endpoint, "LOCAL_GRAPL_REPLACE_IP", attr.unique.network.ip-address)
  redis_endpoint = replace(var._redis_endpoint, "LOCAL_GRAPL_REPLACE_IP", attr.unique.network.ip-address)
  kafka_endpoint = replace(var._kafka_endpoint, "LOCAL_GRAPL_REPLACE_IP", attr.unique.network.ip-address)

  _redis_trimmed = trimprefix(local.local_redis_endpoint, "redis://")
  _redis         = split(":", local._redis_trimmed)
  redis_host     = local._redis[0]
  redis_port     = local._redis[1]
}

job "integration-tests" {
  datacenters = ["dc1"]
  type        = "batch"
  parameterized {}

  reschedule {
    # Make this a one-shot job
    attempts = 0
  }

  # Specifies that this job is the most high priority job we have; nothing else should take precedence 
  priority = 100

  group "rust-integration-tests" {
    restart {
      # Make this a one-shot job
      attempts = 0
    }

    network {
      mode = "bridge"
    }

    # Enable service discovery
    service {
      name = "rust-integration-tests"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              # This is a hack, because IDK how to share locals across files
              destination_name = "dgraph-alpha-0-grpc-public"
              local_bind_port  = 9080
            }
          }
        }
      }
    }

    task "rust-integration-tests" {
      driver = "docker"

      config {
        image = "${var.container_registry}grapl/rust-integration-tests:dev"
      }

      env {
        AWS_REGION                  = var.aws_region
        DEPLOYMENT_NAME             = var.deployment_name
        GRAPL_AWS_ENDPOINT          = local.local_aws_endpoint
        GRAPL_AWS_ACCESS_KEY_ID     = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET = var.aws_access_key_secret
        GRAPL_LOG_LEVEL             = local.log_level
        # This is a hack, because IDK how to share locals across files
        #MG_ALPHAS                   = local.alpha_grpc_connect_str # TODO: Figure out how to do this
        MG_ALPHAS      = "localhost:9080"
        RUST_BACKTRACE = 1
        RUST_LOG       = local.log_level
        REDIS_ENDPOINT = local.local_redis_endpoint
        KAFKA_ENDPOINT = local.local_kafka_endpoint
      }
    }
  }
  group "analyzerlib-integration-tests" {
    restart {
      # Make this a one-shot job
      attempts = 0
    }

    network {
      mode = "bridge"
    }

    # Enable service discovery
    service {
      name = "analyzerlib-integration-tests"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              # This is a hack, because IDK how to share locals across files
              destination_name = "dgraph-alpha-0-grpc-public"
              local_bind_port  = 9080
            }
          }
        }
      }
    }

    task "analyzerlib-integration-tests" {
      driver = "docker"

      config {
        image      = "${var.container_registry}grapl/analyzerlib-test:dev"
        entrypoint = ["/bin/bash", "-o", "errexit", "-o", "nounset", "-c"]
        command    = "cd grapl_analyzerlib && py.test -v -n auto -m 'integration_test'"
      }

      env {
        DEPLOYMENT_NAME = var.deployment_name
        GRAPL_LOG_LEVEL = local.log_level
        MG_ALPHAS       = "localhost:9080"
      }

      resources {
        cpu    = 500
        memory = 1024
      }

    }
  }

  group "analyzer-executor-integration-tests" {
    restart {
      # Make this a one-shot job
      attempts = 0
    }

    network {
      mode = "bridge"
    }

    # Enable service discovery
    service {
      name = "analyzer-executor-integration-tests"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              # This is a hack, because IDK how to share locals across files
              destination_name = "dgraph-alpha-0-grpc-public"
              local_bind_port  = 9080
            }
          }
        }
      }
    }
    task "analyzer-executor-integration-tests" {
      driver = "docker"

      config {
        image      = "${var.container_registry}grapl/analyzer-executor-test:dev"
        entrypoint = ["/bin/bash", "-o", "errexit", "-o", "nounset", "-c"]
        command    = "cd analyzer_executor && export PYTHONPATH=\"$(pwd)/src\"; py.test -n auto -m 'integration_test'"
      }

      env {
        # aws vars
        AWS_REGION                  = var.aws_region
        GRAPL_AWS_ENDPOINT          = local.local_aws_endpoint
        GRAPL_AWS_ACCESS_KEY_ID     = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET = var.aws_access_key_secret

        GRAPL_LOG_LEVEL = local.log_level

        # These environment vars need to exist but the values aren't actually exercised
        GRAPL_ANALYZER_MATCHED_SUBGRAPHS_BUCKET = "NOT_ACTUALLY_EXERCISED_IN_TESTS"
        GRAPL_ANALYZERS_BUCKET                  = "NOT_ACTUALLY_EXERCISED_IN_TESTS"
        GRAPL_MODEL_PLUGINS_BUCKET              = "NOT_ACTUALLY_EXERCISED_IN_TESTS"

        HITCACHE_ADDR     = local.redis_host
        HITCACHE_PORT     = local.redis_port
        MESSAGECACHE_ADDR = local.redis_host
        MESSAGECACHE_PORT = local.redis_port
        IS_RETRY          = false
      }

      resources {
        cpu    = 500
        memory = 1024
      }

    }
  }

  group "graphql-endpoint-tests" {
    restart {
      # Make this a one-shot job
      attempts = 0
    }

    network {
      mode = "bridge"
    }

    # Enable service discovery
    service {
      name = "graphql-endpoint-tests"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              # This is a hack, because IDK how to share locals across files
              destination_name = "dgraph-alpha-0-grpc-public"
              local_bind_port  = 9080
            }
          }
        }
      }
    }
    task "graphql-endpoint-tests" {
      driver = "docker"

      config {
        image      = "${var.container_registry}grapl/graphql-endpoint-tests:dev"
        entrypoint = ["/bin/bash", "-o", "errexit", "-o", "nounset", "-c"]
        command    = "cd graphql_endpoint_tests; py.test --capture=no -n 1 -m 'integration_test'"
      }

      env {
        # aws vars
        AWS_REGION                  = var.aws_region
        GRAPL_AWS_ENDPOINT          = local.local_aws_endpoint
        GRAPL_AWS_ACCESS_KEY_ID     = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET = var.aws_access_key_secret

        DEPLOYMENT_NAME = var.deployment_name
        GRAPL_LOG_LEVEL = local.log_level

        # These are placeholders since Ian is replacing the nginx service shortly
        GRAPL_API_HOST           = "localhost"
        GRAPL_HTTP_FRONTEND_PORT = 3128
        GRAPL_TEST_USER_NAME     = ""

        IS_LOCAL  = true
        MG_ALPHAS = "localhost:9080"
      }
    }


  }

  group "engagement-edge-integration-tests" {
    restart {
      # Make this a one-shot job
      attempts = 0
    }

    network {
      mode = "bridge"
    }

    # Enable service discovery
    service {
      name = "engagement-edge-integration-tests"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              # This is a hack, because IDK how to share locals across files
              destination_name = "dgraph-alpha-0-grpc-public"
              local_bind_port  = 9080
            }
          }
        }
      }
    }

    task "engagement-edge-integration-tests" {
      driver = "docker"

      config {
        image      = "${var.container_registry}grapl/grapl-engagement-edge-test:dev"
        entrypoint = ["/bin/bash", "-o", "errexit", "-o", "nounset", "-c"]
        command    = "cd engagement_edge; py.test -n auto -m 'integration_test'"
      }

      env {
        # aws vars
        AWS_REGION                  = var.aws_region
        GRAPL_AWS_ENDPOINT          = local.local_aws_endpoint
        GRAPL_AWS_ACCESS_KEY_ID     = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET = var.aws_access_key_secret

        DEPLOYMENT_NAME = var.deployment_name
        GRAPL_LOG_LEVEL = local.log_level

        # These are placeholders since Ian is replacing the nginx service shortly
        GRAPL_API_HOST           = "localhost"
        GRAPL_HTTP_FRONTEND_PORT = 3128
        GRAPL_TEST_USER_NAME     = ""

        IS_LOCAL  = true
        MG_ALPHAS = "localhost:9080"

        UX_BUCKET_URL           = "ux_bucket_url"
        GRAPL_NOTEBOOK_INSTANCE = "local-grapl-Notebook"
      }

      resources {
        cpu    = 500
        memory = 1024
      }
    }


  }
}

