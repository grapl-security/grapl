variable "container_registry" {
  type        = string
  default     = "localhost:5000"
  description = "The container registry in which we can find Grapl services."
}

# The following variables are all-caps to clue in users that they're
# imported from `local-grapl.env`.
variable "KAFKA_BROKER_HOST" {
  type        = string
  description = "External hostname for kafka"
}

variable "KAFKA_BROKER_PORT" {
  type        = string
  description = "Port for kafka broker"
}

variable "KAFKA_JMX_PORT" {
  type        = string
  description = "Port for kafka JMX"
}

variable "FAKE_AWS_ACCESS_KEY_ID" {
  type        = string
  description = "Fake AWS Access Key ID for Localstack and clients"
}

variable "FAKE_AWS_SECRET_ACCESS_KEY" {
  type        = string
  description = "Fake AWS Secret Access Key for Localstack and clients"
}

variable "LOCALSTACK_PORT" {
  type        = string
  description = "Port for Localstack"
}

variable "LOCALSTACK_HOST" {
  type        = string
  description = "External hostname for Localstack"
}

variable "ZOOKEEPER_HOST" {
  type        = string
  description = "External hostname for zookeeper"
}

variable "ZOOKEEPER_PORT" {
  type        = string
  description = "Port for zookeeper"
}


####################
# Jobspecs
####################

# This job is to spin up infrastructure needed to run Grapl locally (e.g. Redis) that we don't necessarily want to deploy in production (because AWS will manage it)
job "grapl-local-infra" {
  datacenters = ["dc1"]

  type = "service"

  group "redis" {
    network {
      port "redis" {
        static = 6379
      }
    }

    task "redis" {
      driver = "docker"

      config {
        image        = "redis:latest"
        ports        = ["redis"]
        network_mode = "grapl-network"
        network_aliases = [
          # TODO: import as var
          "redis.grapl.test",
        ]
      }
    }
  }

  group "localstack" {
    network {
      port "localstack" {
        to = var.LOCALSTACK_PORT
      }
    }

    task "localstack" {
      driver = "docker"

      config {
        # Once we move to Kafka, we can go back to the non-fork.
        image = "${var.container_registry}/grapl/localstack:latest"
        # Was running into this: https://github.com/localstack/localstack/issues/1349
        memory_hard_limit = 2048
        ports             = ["localstack"]
        privileged        = true
        volumes = [
          "/var/run/docker.sock:/var/run/docker.sock"
        ]
        network_mode = "grapl-network"
        network_aliases = [
          var.LOCALSTACK_HOST
        ]
      }

      env {
        DEBUG             = 1
        EDGE_PORT         = var.LOCALSTACK_PORT
        HOSTNAME_EXTERNAL = var.LOCALSTACK_HOST
        LAMBDA_EXECUTOR   = "docker-reuse"
        SERVICES          = "apigateway,cloudwatch,dynamodb,ec2,events,iam,lambda,logs,s3,secretsmanager,sns,sqs"
        SQS_PROVIDER      = "elasticmq"

        # These two are only required for Lambda support.
        # Container name is *not* configurable.
        MAIN_CONTAINER_NAME   = "${NOMAD_TASK_NAME}-${NOMAD_ALLOC_ID}"
        LAMBDA_DOCKER_NETWORK = "grapl-network"

        # These are not used by localstack, but are used by the health check.
        AWS_ACCESS_KEY_ID     = var.FAKE_AWS_ACCESS_KEY_ID
        AWS_SECRET_ACCESS_KEY = var.FAKE_AWS_SECRET_ACCESS_KEY
      }

      service {
        check {
          type    = "script"
          name    = "check_s3_ls"
          command = "/bin/bash"
          args = [
            "-c",
            "aws --endpoint-url=http://localhost:${EDGE_PORT} s3 ls",
          ]
          interval = "20s"
          timeout  = "10s"

          check_restart {
            limit           = 2
            grace           = "30s"
            ignore_warnings = false
          }
        }
      }

    }
  }

  group "ratel" {
    network {
      port "ratel" {
        static = 8000
      }
    }

    task "ratel" {
      driver = "docker"

      config {
        image = "dgraph/ratel:latest"
        ports = ["ratel"]
      }
    }
  }

  group "kafka" {
    network {
      mode = "bridge"

      port "kafka" {
        to = var.KAFKA_BROKER_PORT
      }
    }

    task "kafka" {
      driver = "docker"

      config {
        image = "confluentinc/cp-kafka:6.2.0"
        ports = ["kafka"]
        network_aliases = [
          var.KAFKA_BROKER_HOST
        ]
      }

      env {
        KAFKA_BROKER_ID                                = 1
        KAFKA_ZOOKEEPER_CONNECT                        = "${var.ZOOKEEPER_HOST}:${var.ZOOKEEPER_PORT}"
        KAFKA_LISTENER_SECURITY_PROTOCOL_MAP           = "PLAINTEXT:PLAINTEXT"
        KAFKA_LISTENERS                                = "PLAINTEXT://${var.KAFKA_BROKER_HOST}:${var.KAFKA_BROKER_PORT}"
        KAFKA_ADVERTISED_LISTENERS                     = "PLAINTEXT://${var.KAFKA_BROKER_HOST}:${var.KAFKA_BROKER_PORT}"
        KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR         = 1
        KAFKA_TRANSACTION_STATE_LOG_MIN_ISR            = 1
        KAFKA_TRANSACTION_STATE_LOG_REPLICATION_FACTOR = 1
        KAFKA_GROUP_INITIAL_REBALANCE_DELAY_MS         = 0
        KAFKA_JMX_PORT                                 = var.KAFKA_JMX_PORT
        KAFKA_JMX_HOSTNAME                             = "localhost"
        KAFKA_LOG4J_ROOT_LOGLEVEL                      = "INFO"

      }
    }
  }

  group "zookeeper" {
    network {
      mode = "bridge"

      port "zookeeper" {
        to = var.ZOOKEEPER_PORT
      }
    }

    task "zookeeper" {
      driver = "docker"

      config {
        image = "confluentinc/cp-zookeeper:6.2.0"
        ports = ["zookeeper"]
        network_aliases = [
          var.ZOOKEEPER_HOST
        ]
      }

      env {
        ZOOKEEPER_CLIENT_PORT = var.ZOOKEEPER_PORT
        ZOOKEEPER_TICK_TIME   = 2000
      }
    }
  }
}