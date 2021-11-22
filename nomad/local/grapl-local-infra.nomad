variable "container_repository" {
  type        = string
  default     = ""
  description = "The container repository in which we can find Grapl services. Requires a trailing /"
}

variable "localstack_tag" {
  type        = string
  description = "The tagged version of localstack we should deploy."
}

# The following variables are all-caps to clue in users that they're
# imported from `local-grapl.env`.
variable "KAFKA_BROKER_PORT" {
  type        = string
  description = "Kafka Broker's port to listen on, for other Nomad clients"
}

variable "KAFKA_BROKER_PORT_FOR_HOST_OS" {
  type        = string
  description = "Kafka Broker's port to listen on, for things on the host OS (like Pulumi)"
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

variable "ZOOKEEPER_PORT" {
  type        = string
  description = "Port for zookeeper"
}

locals {
  # This is the equivalent of `localhost` within a bridge network.
  # Useful for, for instance, talking to Zookeeper from Kafka without Consul Connect
  localhost_within_bridge = attr.unique.network.ip-address
  zookeeper_endpoint      = "${local.localhost_within_bridge}:${var.ZOOKEEPER_PORT}"
}


####################
# Jobspecs
####################
# NOTES:
# - Services in `grapl-core.nomad` should not try to service-discover
#   local-infra services via Consul Connect; use bridge+static.
#   This is because these services won't exist in prod.

# This job is to spin up infrastructure needed to run Grapl locally (e.g. Redis) that we don't necessarily want to deploy in production (because AWS will manage it)
job "grapl-local-infra" {
  datacenters = ["dc1"]

  type = "service"

  group "redis" {
    # Redis will be available to Nomad Jobs (sans Consul Connect)
    # and the Host OS at localhost:6379
    network {
      mode = "bridge"
      port "redis" {
        static = 6379
      }
    }

    task "redis" {
      driver = "docker"

      config {
        image = "redis:latest"
        ports = ["redis"]
      }

      service {
        name = "redis"

        check {
          type    = "script"
          name    = "check_redis"
          command = "/bin/bash"
          # Interpolated by bash, not nomad
          args = [
            "-o", "errexit", "-o", "nounset",
            "-c",
            "redis-cli ping || exit 1",
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

  group "localstack" {
    # Localstack will be available to Nomad Jobs (sans Consul Connect)
    # and the Host OS at localhost:4566
    network {
      mode = "bridge"
      port "localstack" {
        static = var.LOCALSTACK_PORT
      }
    }

    task "localstack" {
      driver = "docker"

      config {
        # Once we move to Kafka, we can go back to the non-fork.
        image = "localstack-grapl-fork:${var.localstack_tag}"
        # Was running into this: https://github.com/localstack/localstack/issues/1349
        memory_hard_limit = 2048
        ports             = ["localstack"]
        privileged        = true
        volumes = [
          "/var/run/docker.sock:/var/run/docker.sock"
        ]
      }

      env {
        DEBUG           = 1
        EDGE_PORT       = var.LOCALSTACK_PORT
        LAMBDA_EXECUTOR = "docker-reuse"
        SERVICES        = "apigateway,cloudwatch,dynamodb,ec2,events,iam,lambda,logs,s3,secretsmanager,sns,sqs"
        SQS_PROVIDER    = "elasticmq"

        # These two are only required for Lambda support.
        # Container name is *not* configurable.
        MAIN_CONTAINER_NAME = "${NOMAD_TASK_NAME}-${NOMAD_ALLOC_ID}"

        # These are not used by localstack, but are used by the health check.
        AWS_ACCESS_KEY_ID     = var.FAKE_AWS_ACCESS_KEY_ID
        AWS_SECRET_ACCESS_KEY = var.FAKE_AWS_SECRET_ACCESS_KEY
      }

      service {
        name = "localstack"
        check {
          type    = "script"
          name    = "check_s3_ls"
          command = "/bin/bash"
          args = [
            "-o", "errexit", "-o", "nounset",
            "-c",
            # This uses the stuff in env { } - not Nomad interpolation.
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
      mode = "bridge"
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

      service {
        name = "ratel"
      }
    }
  }

  group "kafka" {
    network {
      mode = "bridge"
      port "kafka-for-other-nomad-tasks" {
        static = var.KAFKA_BROKER_PORT
      }
      port "kafka-for-host-os" {
        static = var.KAFKA_BROKER_PORT_FOR_HOST_OS
      }
    }

    task "kafka" {
      driver = "docker"

      config {
        image = "confluentinc/cp-kafka:6.2.0"
        ports = ["kafka-for-other-nomad-tasks", "kafka-for-host-os"]
      }

      resources {
        cpu    = 500
        memory = 1024
      }

      env {
        KAFKA_BROKER_PORT       = 9092 # Only used by healthcheck
        KAFKA_BROKER_ID         = 1
        KAFKA_ZOOKEEPER_CONNECT = local.zookeeper_endpoint

        # Some clients (like Pulumi) will need `host.docker.internal`
        # Some clients (like grapl-core services) will need localhost_within_bridge
        # We differentiate between which client it is based on which port we receive on. 
        # So a receive on 29092 means HOST_OS
        KAFKA_ADVERTISED_LISTENERS = join(",", [
          "WITHIN_TASK://localhost:9092",
          "HOST_OS://host.docker.internal:${var.KAFKA_BROKER_PORT_FOR_HOST_OS}",
          "OTHER_NOMADS://${local.localhost_within_bridge}:${var.KAFKA_BROKER_PORT}"
        ])
        KAFKA_LISTENER_SECURITY_PROTOCOL_MAP = "WITHIN_TASK:PLAINTEXT,HOST_OS:PLAINTEXT,OTHER_NOMADS:PLAINTEXT"
        KAFKA_INTER_BROKER_LISTENER_NAME     = "WITHIN_TASK"

        KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR         = 1
        KAFKA_TRANSACTION_STATE_LOG_MIN_ISR            = 1
        KAFKA_TRANSACTION_STATE_LOG_REPLICATION_FACTOR = 1
        KAFKA_GROUP_INITIAL_REBALANCE_DELAY_MS         = 0
        KAFKA_JMX_PORT                                 = var.KAFKA_JMX_PORT
        KAFKA_JMX_HOSTNAME                             = "localhost"
        KAFKA_LOG4J_ROOT_LOGLEVEL                      = "INFO"
      }

      service {
        name = "kafka"
        check {
          type    = "script"
          name    = "check_kafka"
          command = "/bin/bash"
          # Interpolated by bash, not nomad
          args = [
            "-o", "errexit", "-o", "nounset",
            "-c",
            "nc -vz localhost ${KAFKA_BROKER_PORT}",
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

  group "zookeeper" {
    network {
      mode = "bridge"
      port "zookeeper" {
        static = var.ZOOKEEPER_PORT
        to     = var.ZOOKEEPER_PORT
      }
    }

    task "zookeeper" {
      driver = "docker"

      config {
        image = "confluentinc/cp-zookeeper:6.2.0"
        ports = ["zookeeper"] # may not be necesary
      }

      env {
        ZOOKEEPER_CLIENT_PORT = var.ZOOKEEPER_PORT
        ZOOKEEPER_TICK_TIME   = 2000
        KAFKA_OPTS            = "-Dzookeeper.4lw.commands.whitelist=ruok,dump"
      }

      service {
        name = "zookeeper"
        check {
          type    = "script"
          name    = "check_zookeeper"
          command = "/bin/bash"
          # Interpolated by bash, not nomad
          args = [
            "-o", "errexit", "-o", "nounset",
            "-c",
            "echo ruok | nc -w 2  localhost ${ZOOKEEPER_CLIENT_PORT} | grep imok || exit 2",
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
}