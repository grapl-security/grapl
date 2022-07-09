variable "image_tag" {
  type        = string
  description = "The tag for all container images we should deploy. This is ultimately set in the top-level Makefile."
}

variable "kafka_broker_port" {
  type        = number
  description = "Kafka Broker's port to listen on, for other Nomad clients"
  default     = 19092
}

variable "kafka_broker_port_for_host_os" {
  type        = number
  description = "Kafka Broker's port to listen on, for things on the host OS (like Pulumi)"
  default     = 29092
}

variable "kafka_jmx_port" {
  type        = number
  description = "Port for Kafka JMX"
  default     = 9101
}

variable "localstack_port" {
  type        = number
  description = "Port for Localstack"
  default     = 4566
}

variable "zookeeper_port" {
  type        = number
  description = "Port for Zookeeper"
  default     = 2181
}

# These Postgres connection data must match what's in
# `pulumi/grapl/__main__.py`; sorry for the duplication :(
variable plugin_registry_db {
  description = "Connection configuration for the Plugin Registry database"
  type = object({
    username = string
    password = string
    port     = number
  })
  default = {
    username = "postgres"
    password = "postgres"
    port     = 5432
  }
}

variable plugin_work_queue_db {
  description = "Connection configuration for the Plugin Work Queue database"
  type = object({
    username = string
    password = string
    port     = number
  })
  default = {
    username = "postgres"
    password = "postgres"
    port     = 5532
  }
}

variable organization_management_db {
  description = "Connection configuration for the Organization Management database"
  type = object({
    username = string
    password = string
    port     = number
  })
  default = {
    username = "postgres"
    password = "postgres"
    port     = 5632
  }
}

variable uid_allocator_db {
  description = "Connection configuration for the Uid Allocator database"
  type = object({
    username = string
    password = string
    port     = number
  })
  default = {
    username = "postgres"
    password = "postgres"
    port     = 5732
  }
}

variable schema_manager_db {
  description = "Connection configuration for the Schema Manager database"
  type = object({
    username = string
    password = string
    port     = number
  })
  default = {
    username = "postgres"
    password = "postgres"
    port     = 5832
  }
}

locals {
  # This is the equivalent of `localhost` within a bridge network.
  # Useful for, for instance, talking to Zookeeper from Kafka without Consul Connect
  localhost_within_bridge = attr.unique.network.ip-address
  zookeeper_endpoint      = "${local.localhost_within_bridge}:${var.zookeeper_port}"

  # These Postgres connection data must match the `LocalPostgresInstance`s in
  # `pulumi/grapl/__main__.py`; sorry for the duplication :(
  database_descriptors = [
    {
      name = "plugin-registry-db",
      port = 5432,
    },
    {
      name = "plugin-work-queue-db",
      port = 5433,
    },
    {
      name = "organization-management-db",
      port = 5434,
    },
    {
      name = "uid-allocator-db",
      port = 5435
    },
    {
      name = "event-source-db",
      port = 5436
    },
  ]
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
          type     = "script"
          name     = "check_redis"
          command  = "redis-cli"
          args     = ["ping"]
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
        static = var.localstack_port
      }
    }

    task "localstack" {
      driver = "docker"

      config {
        # https://github.com/localstack/localstack/issues/5824
        # The bugfix we need is only available post-14.3 in latest starting May 23
        # Hence pinning by sha, not tag
        image = "localstack/localstack-light@sha256:a64dbc0b4e05f3647d8f1a09eb743e3d213402312858fb4146a7571a4a4ee6be"

        # Was running into this: https://github.com/localstack/localstack/issues/1349
        memory_hard_limit = 2048
        ports             = ["localstack"]
        privileged        = true
      }

      env {
        DEBUG     = 1
        EDGE_PORT = var.localstack_port
        SERVICES  = "dynamodb,ec2,iam,s3,secretsmanager,sns"

        # These are used by the health check below; "test" is the
        # default value for these credentials in Localstack.
        AWS_ACCESS_KEY_ID     = "test"
        AWS_SECRET_ACCESS_KEY = "test"
      }

      service {
        name = "localstack"
        check {
          type    = "script"
          name    = "check_s3_ls"
          command = "aws"
          args = [
            "--endpoint-url=http://localhost:${var.localstack_port}",
            "s3",
            "ls"
          ]
          interval = "10s"
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
        static = var.kafka_broker_port
      }
      port "kafka-for-host-os" {
        static = var.kafka_broker_port_for_host_os
      }
    }

    task "kafka" {
      driver = "docker"

      config {
        image = "confluentinc/cp-kafka:7.0.1"
        ports = ["kafka-for-other-nomad-tasks", "kafka-for-host-os"]
      }

      resources {
        memory = 500
      }

      env {
        kafka_broker_port       = 9092 # Only used by healthcheck
        KAFKA_BROKER_ID         = 1
        KAFKA_ZOOKEEPER_CONNECT = local.zookeeper_endpoint

        # Some clients (like Pulumi) will need `host.docker.internal`
        # Some clients (like grapl-core services) will need localhost_within_bridge
        # We differentiate between which client it is based on which port we receive on.
        # So a receive on 29092 means HOST_OS
        KAFKA_ADVERTISED_LISTENERS = join(",", [
          "WITHIN_TASK://localhost:9092",
          "HOST_OS://host.docker.internal:${var.kafka_broker_port_for_host_os}",
          "OTHER_NOMADS://${local.localhost_within_bridge}:${var.kafka_broker_port}"
        ])
        KAFKA_AUTO_CREATE_TOPICS_ENABLE      = "false"
        KAFKA_LISTENER_SECURITY_PROTOCOL_MAP = "WITHIN_TASK:PLAINTEXT,HOST_OS:PLAINTEXT,OTHER_NOMADS:PLAINTEXT"
        KAFKA_INTER_BROKER_LISTENER_NAME     = "WITHIN_TASK"

        KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR         = 1
        KAFKA_TRANSACTION_STATE_LOG_MIN_ISR            = 1
        KAFKA_TRANSACTION_STATE_LOG_REPLICATION_FACTOR = 1
        KAFKA_GROUP_INITIAL_REBALANCE_DELAY_MS         = 0
        KAFKA_JMX_PORT                                 = var.kafka_jmx_port
        KAFKA_JMX_HOSTNAME                             = "localhost"
        KAFKA_LOG4J_ROOT_LOGLEVEL                      = "INFO"
      }

      service {
        name = "kafka"
        check {
          type    = "script"
          name    = "check_kafka"
          command = "nc"
          args = [
            "-v", # verbose
            "-z", # "zero I/O mode" - used for scanning
            "localhost",
            "${var.kafka_broker_port}"
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
        static = var.zookeeper_port
        to     = var.zookeeper_port
      }
    }

    task "zookeeper" {
      driver = "docker"

      config {
        image = "confluentinc/cp-zookeeper:7.0.1"
        ports = ["zookeeper"] # may not be necessary
      }

      env {
        ZOOKEEPER_CLIENT_PORT = var.zookeeper_port
        ZOOKEEPER_TICK_TIME   = 2000
        KAFKA_OPTS            = "-Dzookeeper.4lw.commands.whitelist=ruok,dump"
      }

      service {
        name = "zookeeper"
        check {
          type    = "script"
          name    = "check_zookeeper"
          command = "/bin/bash"
          args = [
            "-o", "errexit", "-o", "nounset", "-o", "pipefail",
            "-c",
            "echo ruok | nc -w 2 localhost ${var.zookeeper_port} | grep imok || exit 2",
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

  # Construct N groups for each entry in database_descriptors,
  # each one containing a Postgres task.
  dynamic "group" {
    for_each = local.database_descriptors
    iterator = db_desc

    labels = [db_desc.value.name]

    content {
      network {
        mode = "bridge"
        port "postgres" {
          static = db_desc.value.port
          to     = 5432 # postgres default
        }
      }

      # This is a hack so that the task name can be something dynamic.
      # (In this case, each task has the same name as the group.)
      # I do this because otherwise we'd have N logs called 'postgres.stdout'
      # It is for-each over a list with a single element: [db_desc].
      dynamic "task" {
        for_each = [db_desc.value]
        iterator = db_desc

        labels = [db_desc.value.name]

        content {
          driver = "docker"

          config {
            image = "postgres-ext:${var.image_tag}"
            ports = ["postgres"]
          }

          env {
            POSTGRES_USER     = "postgres"
            POSTGRES_PASSWORD = "postgres"
          }

          service {
            name = db_desc.value.name

            check {
              type     = "script"
              name     = "check_postgres"
              command  = "pg_isready"
              args     = ["--username", "postgres"]
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
  }

  group "dnsmasq" {
    network {
      mode = "bridge"
      port "dns" {
        static = 53
        to     = 53
      }
    }


    task "dnsmasq" {
      driver = "docker"

      config {
        #This is an alpine-based dnsmasq container
        image = "4km3/dnsmasq:2.85-r2"
        ports = ["dns"]
        args = [
          # Send all queries for .consul to the NOMAD_IP
          "--server", "/consul/${NOMAD_IP_dns}#8600",
          # log to standard out
          "--log-facility=-",
        ]
        cap_add = [
          "NET_BIND_SERVICE",
        ]
        logging {
          type = "journald"
          config {
            tag = "DNSMASQ"
          }
        }
      }

      service {
        name         = "dnsmasq"
        port         = "dns"
        address_mode = "driver"
        tags         = ["dns"]

        check {
          type     = "tcp"
          port     = "dns"
          interval = "10s"
          timeout  = "2s"
        }
      }

      resources {
        cpu    = 50
        memory = 100
      }
    }
  }

  group "organization-management-db" {
    network {
      mode = "bridge"
      port "postgres" {
        static = var.organization_management_db.port
        to     = 5432
      }
    }

    task "organization-management-db" {
      driver = "docker"

      config {
        image = "postgres-ext:${var.image_tag}"
        ports = ["postgres"]
      }

      env {
        POSTGRES_USER     = var.organization_management_db.username
        POSTGRES_PASSWORD = var.organization_management_db.password
      }

      service {
        name = "organization-management-db"

        check {
          type     = "script"
          name     = "check_postgres"
          command  = "pg_isready"
          args     = ["--username", "${var.organization_management_db.username}"]
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

  group "uid-allocator-db" {
    network {
      mode = "bridge"
      port "postgres" {
        static = var.uid_allocator_db.port
        to     = 5432
      }
    }

    task "uid-allocator-db" {
      driver = "docker"

      config {
        image = "postgres-ext:${var.image_tag}"
        ports = ["postgres"]
      }

      env {
        POSTGRES_USER     = var.uid_allocator_db.username
        POSTGRES_PASSWORD = var.uid_allocator_db.password
      }

      service {
        name = "uid-allocator-db"

        check {
          type     = "script"
          name     = "check_postgres"
          command  = "pg_isready"
          args     = ["--username", "${var.uid_allocator_db.username}"]
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

  group "schema-manager-db" {
    network {
      mode = "bridge"
      port "postgres" {
        static = var.schema_manager_db.port
        to     = 5432
      }
    }

    task "schema-manager-db" {
      driver = "docker"

      config {
        image = "postgres-ext:${var.image_tag}"
        ports = ["postgres"]
      }

      env {
        POSTGRES_USER     = var.schema_manager_db.username
        POSTGRES_PASSWORD = var.schema_manager_db.password
      }

      service {
        name = "schema-manager-db"

        check {
          type     = "script"
          name     = "check_postgres"
          command  = "pg_isready"
          args     = ["--username", "${var.schema_manager_db.username}"]
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

  group "scylla" {
    network {
      mode = "bridge"
      port "internal_node_rpc_1" {
        to = 7000
      }
      port "internal_node_rpc_2" {
        to = 7001
      }
      port "cql" {
        # Let devs connect via localhost:9042 from the host vm
        static = 9042
        to     = 9042
      }
      port "thrift" {
        to = 9160
      }
      port "rest" {
        to = 10000
      }
    }

    task "scylla" {
      driver = "docker"

      config {
        image = "scylladb-ext:${var.image_tag}"
        args = [
          # Set up scylla in single-node mode instead of in overprovisioned mode, ie DON'T use all available cpu/memory
          "--smp", "1"
        ]
        ports = ["internal_node_rpc_1", "internal_node_rpc_2", "cql", "thrift", "rest"]

        # Configure a data volume for scylla. See the "Configuring data volume for storage" section in
        # https://hub.docker.com/r/scylladb/scylla/
        mount {
          type     = "volume"
          target   = "/var/lib/scylla"
          source   = "scylla-data"
          readonly = false
          volume_options {
            # Upon initial creation of this volume, *do* copy in the current
            # contents in the Docker image.
            no_copy = false
            labels {
              maintainer = "Scylla"
            }
          }
        }

      }

      env {
        # This enables username/password auth.
        AUTHENTICATOR = "CassandraAuthorizer"
      }

      service {
        name = "scylla"

        check {
          type = "script"
          name = "nodestatus_check"
          # We use bin/bash so we can pipe to grep
          command  = "bin/bash"
          args     = ["nodetool", "status", "|", "grep", "'UN'"]
          interval = "30s"
          timeout  = "10s"

          check_restart {
            # Set readiness check since Scylla can take a while to boot up
            grace           = "1m"
            limit           = 3
            ignore_warnings = true
          }

        }

      }
    }
  }
}
