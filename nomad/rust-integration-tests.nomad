# This setup is inspired by the following forum discussion:
# https://discuss.hashicorp.com/t/best-practices-for-testing-against-services-in-nomad-consul-connect/29022

variable "container_images" {
  type        = map(string)
  description = <<EOF
  A map of $NAME_OF_TASK to the URL for that task's docker image ID.
  (See DockerImageId in Pulumi for further documentation).
EOF
}

variable "aws_region" {
  type = string
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

variable "kafka_bootstrap_servers" {
  type        = string
  description = "The URL(s) (possibly comma-separated) of the Kafka bootstrap servers."
}

variable "kafka_consumer_group" {
  type        = string
  description = "The name of the consumer group the integration test consumers will join."
}

variable "kafka_credentials" {
  description = "Kafka credentials for the integration tests"
  type = object({
    # The username to authenticate with Confluent Cloud cluster.
    sasl_username = string
    # The password to authenticate with Confluent Cloud cluster.
    sasl_password = string
  })
}

variable "observability_env_vars" {
  type        = string
  description = <<EOF
With local-grapl, we have to inject env vars for Opentelemetry.
In prod, this is currently disabled.
EOF
}

variable "rust_log" {
  type        = string
  description = "Controls the logging behavior of Rust-based services."
}

variable "user_auth_table" {
  type        = string
  description = "The name of the DynamoDB user auth table"
}

variable "user_session_table" {
  type        = string
  description = "The name of the DynamoDB user session table"
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

variable "plugin_work_queue_db" {
  type = object({
    hostname = string
    port     = number
    username = string
    password = string
  })
  description = "Vars for plugin-work-queue database"
}

locals {
  dns_servers = [attr.unique.network.ip-address]

  app_version                      = split(":", var.container_images["rust-integration-tests"])[1]
  default_otel_resource_attributes = "service.version=${local.app_version},host.hostname=${attr.unique.hostname}"
}

job "rust-integration-tests" {
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
      dns {
        servers = local.dns_servers
      }
    }

    # Enable service discovery
    service {
      name = "rust-integration-tests"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "pipeline-ingress"
              local_bind_port  = 1000
            }

            upstreams {
              destination_name = "plugin-registry"
              local_bind_port  = 1001
            }

            upstreams {
              destination_name = "plugin-work-queue"
              local_bind_port  = 1002
            }

            upstreams {
              destination_name = "organization-management"
              local_bind_port  = 1004
            }

            upstreams {
              destination_name = "event-source"
              local_bind_port  = 1005
            }

            upstreams {
              destination_name = "web-ui"
              local_bind_port  = 1006
            }

            upstreams {
              destination_name = "graph-query"
              local_bind_port  = 1007
            }

            upstreams {
              destination_name = "graph-mutation"
              local_bind_port  = 1008
            }

            upstreams {
              destination_name = "graph-schema-manager"
              local_bind_port  = 1009
            }

            upstreams {
              destination_name = "uid-allocator"
              local_bind_port  = 1010
            }

            upstreams {
              destination_name = "scylla-provisioner"
              local_bind_port  = 1011
            }
          }
        }
      }
    }

    task "rust-integration-tests" {
      driver = "docker"

      config {
        image = var.container_images["rust-integration-tests"]
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      env {
        AWS_REGION = var.aws_region

        RUST_BACKTRACE = 1
        RUST_LOG       = var.rust_log

        OTEL_RESOURCE_ATTRIBUTES = local.default_otel_resource_attributes

        # web-ui
        GRAPL_USER_AUTH_TABLE         = var.user_auth_table
        GRAPL_USER_SESSION_TABLE      = var.user_session_table
        GRAPL_WEB_UI_ENDPOINT_ADDRESS = "http://${NOMAD_UPSTREAM_ADDR_web-ui}"

        ORGANIZATION_MANAGEMENT_BIND_ADDRESS = "0.0.0.0:1004" # not used but required due to clap
        ORGANIZATION_MANAGEMENT_DB_ADDRESS   = "${var.organization_management_db.hostname}:${var.organization_management_db.port}"
        ORGANIZATION_MANAGEMENT_DB_PASSWORD  = var.organization_management_db.password
        ORGANIZATION_MANAGEMENT_DB_USERNAME  = var.organization_management_db.username

        ORGANIZATION_MANAGEMENT_HEALTHCHECK_POLLING_INTERVAL_MS = 5000

        EVENT_SOURCE_CLIENT_ADDRESS                       = "http://${NOMAD_UPSTREAM_ADDR_event-source}"
        EVENT_SOURCE_CLIENT_REQUEST_TIMEOUT               = "1s"
        EVENT_SOURCE_CLIENT_EXECUTOR_TIMEOUT              = "1s"
        EVENT_SOURCE_CLIENT_CONCURRENCY_LIMIT             = 16
        EVENT_SOURCE_CLIENT_INITIAL_BACKOFF_DELAY         = "10ms"
        EVENT_SOURCE_CLIENT_MAXIMUM_BACKOFF_DELAY         = "5s"
        EVENT_SOURCE_CLIENT_CONNECT_TIMEOUT               = "5s"
        EVENT_SOURCE_CLIENT_CONNECT_RETRIES               = 10
        EVENT_SOURCE_CLIENT_CONNECT_INITIAL_BACKOFF_DELAY = "1s"
        EVENT_SOURCE_CLIENT_CONNECT_MAXIMUM_BACKOFF_DELAY = "60s"

        GRAPH_QUERY_CLIENT_ADDRESS                       = "http://${NOMAD_UPSTREAM_ADDR_graph-query}"
        GRAPH_QUERY_CLIENT_REQUEST_TIMEOUT               = "1s"
        GRAPH_QUERY_CLIENT_EXECUTOR_TIMEOUT              = "1s"
        GRAPH_QUERY_CLIENT_CONCURRENCY_LIMIT             = 16
        GRAPH_QUERY_CLIENT_INITIAL_BACKOFF_DELAY         = "10ms"
        GRAPH_QUERY_CLIENT_MAXIMUM_BACKOFF_DELAY         = "5s"
        GRAPH_QUERY_CLIENT_CONNECT_TIMEOUT               = "5s"
        GRAPH_QUERY_CLIENT_CONNECT_RETRIES               = 10
        GRAPH_QUERY_CLIENT_CONNECT_INITIAL_BACKOFF_DELAY = "1s"
        GRAPH_QUERY_CLIENT_CONNECT_MAXIMUM_BACKOFF_DELAY = "60s"

        GRAPH_MUTATION_CLIENT_ADDRESS                       = "http://${NOMAD_UPSTREAM_ADDR_graph-mutation}"
        GRAPH_MUTATION_CLIENT_REQUEST_TIMEOUT               = "1s"
        GRAPH_MUTATION_CLIENT_EXECUTOR_TIMEOUT              = "1s"
        GRAPH_MUTATION_CLIENT_CONCURRENCY_LIMIT             = 16
        GRAPH_MUTATION_CLIENT_INITIAL_BACKOFF_DELAY         = "10ms"
        GRAPH_MUTATION_CLIENT_MAXIMUM_BACKOFF_DELAY         = "5s"
        GRAPH_MUTATION_CLIENT_CONNECT_TIMEOUT               = "5s"
        GRAPH_MUTATION_CLIENT_CONNECT_RETRIES               = 10
        GRAPH_MUTATION_CLIENT_CONNECT_INITIAL_BACKOFF_DELAY = "1s"
        GRAPH_MUTATION_CLIENT_CONNECT_MAXIMUM_BACKOFF_DELAY = "60s"

        GRAPH_SCHEMA_MANAGER_CLIENT_ADDRESS                       = "http://${NOMAD_UPSTREAM_ADDR_graph-schema-manager}"
        GRAPH_SCHEMA_MANAGER_CLIENT_REQUEST_TIMEOUT               = "1s"
        GRAPH_SCHEMA_MANAGER_CLIENT_EXECUTOR_TIMEOUT              = "1s"
        GRAPH_SCHEMA_MANAGER_CLIENT_CONCURRENCY_LIMIT             = 16
        GRAPH_SCHEMA_MANAGER_CLIENT_INITIAL_BACKOFF_DELAY         = "10ms"
        GRAPH_SCHEMA_MANAGER_CLIENT_MAXIMUM_BACKOFF_DELAY         = "5s"
        GRAPH_SCHEMA_MANAGER_CLIENT_CONNECT_TIMEOUT               = "5s"
        GRAPH_SCHEMA_MANAGER_CLIENT_CONNECT_RETRIES               = 10
        GRAPH_SCHEMA_MANAGER_CLIENT_CONNECT_INITIAL_BACKOFF_DELAY = "1s"
        GRAPH_SCHEMA_MANAGER_CLIENT_CONNECT_MAXIMUM_BACKOFF_DELAY = "60s"

        ORGANIZATION_MANAGEMENT_CLIENT_ADDRESS                       = "http://${NOMAD_UPSTREAM_ADDR_organization-management}"
        ORGANIZATION_MANAGEMENT_CLIENT_REQUEST_TIMEOUT               = "1s"
        ORGANIZATION_MANAGEMENT_CLIENT_EXECUTOR_TIMEOUT              = "1s"
        ORGANIZATION_MANAGEMENT_CLIENT_CONCURRENCY_LIMIT             = 16
        ORGANIZATION_MANAGEMENT_CLIENT_INITIAL_BACKOFF_DELAY         = "10ms"
        ORGANIZATION_MANAGEMENT_CLIENT_MAXIMUM_BACKOFF_DELAY         = "5s"
        ORGANIZATION_MANAGEMENT_CLIENT_CONNECT_TIMEOUT               = "5s"
        ORGANIZATION_MANAGEMENT_CLIENT_CONNECT_RETRIES               = 10
        ORGANIZATION_MANAGEMENT_CLIENT_CONNECT_INITIAL_BACKOFF_DELAY = "1s"
        ORGANIZATION_MANAGEMENT_CLIENT_CONNECT_MAXIMUM_BACKOFF_DELAY = "60s"

        PIPELINE_INGRESS_CLIENT_ADDRESS                       = "http://${NOMAD_UPSTREAM_ADDR_pipeline-ingress}"
        PIPELINE_INGRESS_CLIENT_REQUEST_TIMEOUT               = "1s"
        PIPELINE_INGRESS_CLIENT_EXECUTOR_TIMEOUT              = "1s"
        PIPELINE_INGRESS_CLIENT_CONCURRENCY_LIMIT             = 16
        PIPELINE_INGRESS_CLIENT_INITIAL_BACKOFF_DELAY         = "10ms"
        PIPELINE_INGRESS_CLIENT_MAXIMUM_BACKOFF_DELAY         = "5s"
        PIPELINE_INGRESS_CLIENT_CONNECT_TIMEOUT               = "5s"
        PIPELINE_INGRESS_CLIENT_CONNECT_RETRIES               = 10
        PIPELINE_INGRESS_CLIENT_CONNECT_INITIAL_BACKOFF_DELAY = "1s"
        PIPELINE_INGRESS_CLIENT_CONNECT_MAXIMUM_BACKOFF_DELAY = "60s"

        PLUGIN_REGISTRY_CLIENT_ADDRESS                       = "http://0.0.0.0:${NOMAD_UPSTREAM_PORT_plugin-registry}"
        PLUGIN_REGISTRY_CLIENT_REQUEST_TIMEOUT               = "1s"
        PLUGIN_REGISTRY_CLIENT_EXECUTOR_TIMEOUT              = "1s"
        PLUGIN_REGISTRY_CLIENT_CONCURRENCY_LIMIT             = 16
        PLUGIN_REGISTRY_CLIENT_INITIAL_BACKOFF_DELAY         = "10ms"
        PLUGIN_REGISTRY_CLIENT_MAXIMUM_BACKOFF_DELAY         = "5s"
        PLUGIN_REGISTRY_CLIENT_CONNECT_TIMEOUT               = "5s"
        PLUGIN_REGISTRY_CLIENT_CONNECT_RETRIES               = 10
        PLUGIN_REGISTRY_CLIENT_CONNECT_INITIAL_BACKOFF_DELAY = "1s"
        PLUGIN_REGISTRY_CLIENT_CONNECT_MAXIMUM_BACKOFF_DELAY = "60s"

        PLUGIN_WORK_QUEUE_CLIENT_ADDRESS                       = "http://${NOMAD_UPSTREAM_ADDR_plugin-work-queue}"
        PLUGIN_WORK_QUEUE_CLIENT_REQUEST_TIMEOUT               = "1s"
        PLUGIN_WORK_QUEUE_CLIENT_EXECUTOR_TIMEOUT              = "1s"
        PLUGIN_WORK_QUEUE_CLIENT_CONCURRENCY_LIMIT             = 16
        PLUGIN_WORK_QUEUE_CLIENT_INITIAL_BACKOFF_DELAY         = "10ms"
        PLUGIN_WORK_QUEUE_CLIENT_MAXIMUM_BACKOFF_DELAY         = "5s"
        PLUGIN_WORK_QUEUE_CLIENT_CONNECT_TIMEOUT               = "5s"
        PLUGIN_WORK_QUEUE_CLIENT_CONNECT_RETRIES               = 10
        PLUGIN_WORK_QUEUE_CLIENT_CONNECT_INITIAL_BACKOFF_DELAY = "1s"
        PLUGIN_WORK_QUEUE_CLIENT_CONNECT_MAXIMUM_BACKOFF_DELAY = "60s"

        SCYLLA_PROVISIONER_CLIENT_ADDRESS                       = "http://${NOMAD_UPSTREAM_ADDR_scylla-provisioner}"
        SCYLLA_PROVISIONER_CLIENT_REQUEST_TIMEOUT               = "1s"
        SCYLLA_PROVISIONER_CLIENT_EXECUTOR_TIMEOUT              = "1s"
        SCYLLA_PROVISIONER_CLIENT_CONCURRENCY_LIMIT             = 16
        SCYLLA_PROVISIONER_CLIENT_INITIAL_BACKOFF_DELAY         = "10ms"
        SCYLLA_PROVISIONER_CLIENT_MAXIMUM_BACKOFF_DELAY         = "5s"
        SCYLLA_PROVISIONER_CLIENT_CONNECT_TIMEOUT               = "5s"
        SCYLLA_PROVISIONER_CLIENT_CONNECT_RETRIES               = 10
        SCYLLA_PROVISIONER_CLIENT_CONNECT_INITIAL_BACKOFF_DELAY = "1s"
        SCYLLA_PROVISIONER_CLIENT_CONNECT_MAXIMUM_BACKOFF_DELAY = "60s"

        UID_ALLOCATOR_CLIENT_ADDRESS                       = "http://${NOMAD_UPSTREAM_ADDR_uid-allocator}"
        UID_ALLOCATOR_CLIENT_REQUEST_TIMEOUT               = "1s"
        UID_ALLOCATOR_CLIENT_EXECUTOR_TIMEOUT              = "1s"
        UID_ALLOCATOR_CLIENT_CONCURRENCY_LIMIT             = 16
        UID_ALLOCATOR_CLIENT_INITIAL_BACKOFF_DELAY         = "10ms"
        UID_ALLOCATOR_CLIENT_MAXIMUM_BACKOFF_DELAY         = "5s"
        UID_ALLOCATOR_CLIENT_CONNECT_TIMEOUT               = "5s"
        UID_ALLOCATOR_CLIENT_CONNECT_RETRIES               = 10
        UID_ALLOCATOR_CLIENT_CONNECT_INITIAL_BACKOFF_DELAY = "1s"
        UID_ALLOCATOR_CLIENT_CONNECT_MAXIMUM_BACKOFF_DELAY = "60s"

        KAFKA_BOOTSTRAP_SERVERS   = var.kafka_bootstrap_servers
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_group
        KAFKA_SASL_USERNAME       = var.kafka_credentials.sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials.sasl_password

        NOMAD_SERVICE_ADDRESS = "${attr.unique.network.ip-address}:4646"

        PLUGIN_WORK_QUEUE_DB_ADDRESS  = "${var.plugin_work_queue_db.hostname}:${var.plugin_work_queue_db.port}"
        PLUGIN_WORK_QUEUE_DB_USERNAME = var.plugin_work_queue_db.username
        PLUGIN_WORK_QUEUE_DB_PASSWORD = var.plugin_work_queue_db.password

        GRAPH_DB_ADDRESSES     = var.graph_db.addresses
        GRAPH_DB_AUTH_PASSWORD = var.graph_db.password
        GRAPH_DB_AUTH_USERNAME = var.graph_db.username
      }

      resources {
        # We need a lot of memory because we load the 150MB
        # /test-fixtures/example-generator
        # into memory
        memory = 1024
      }
    }
  }
}
