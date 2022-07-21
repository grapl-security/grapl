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

variable "observability_env_vars" {
  type        = string
  description = <<EOF
With local-grapl, we have to inject env vars for Opentelemetry.
In prod, this is currently disabled.
EOF
}

variable "aws_region" {
  type = string
}

variable "kafka_bootstrap_servers" {
  type        = string
  description = "The URL(s) (possibly comma-separated) of the Kafka bootstrap servers."
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

# https://github.com/grapl-security/grapl/blob/af6f2c197d52e9941047aab813c30d2cbfd54523/pulumi/infra/dynamodb.py
variable "static_mapping_table_name" {
  type        = string
  description = "What is the name of the static mapping table?"
}


variable "plugin_registry_db" {
  type = object({
    hostname = string
    port     = number
    username = string
    password = string
  })
  description = "Vars for plugin-registry database"
}

variable "plugin_registry_kernel_artifact_url" {
  type        = string
  description = "URL specifying the kernel.tar.gz for the Firecracker VM"
}

variable "plugin_registry_rootfs_artifact_url" {
  type        = string
  description = "URL specifying the rootfs.tar.gz for the Firecracker VM"
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

variable "organization_management_healthcheck_polling_interval_ms" {
  type        = string
  description = "The amount of time to wait between each healthcheck execution."
}

variable "pipeline_ingress_healthcheck_polling_interval_ms" {
  type        = string
  description = "The amount of time to wait between each healthcheck execution."
}

variable "kafka_credentials" {
  description = "Map from service-name to kafka credentials for that service"
  type = map(object({
    # The username to authenticate with Confluent Cloud cluster.
    sasl_username = string
    # The password to authenticate with Confluent Cloud cluster.
    sasl_password = string
  }))
}

variable "kafka_consumer_groups" {
  description = "Map from service-name to the consumer group for that service to join"
  type        = map(string)
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

variable "schema_manager_db" {
  type = object({
    hostname = string
    port     = number
    username = string
    password = string
  })
  description = "Vars for schema-manager database"
}

variable "uid_allocator_db" {
  type = object({
    hostname = string
    port     = number
    username = string
    password = string
  })
  description = "Vars for uid-allocator database"
}

variable "graph_db" {
  type = object({
    addresses = string
    username  = string
    password  = string
  })
  description = "Vars for graph (scylla) database"
}

variable "uid_allocator_service_config" {
  type = object({
    default_allocation_size = number
    preallocation_size      = number
    maximum_allocation_size = number
  })
  description = "Vars for the uid allocator service"
}

variable "event_source_db" {
  type = object({
    hostname = string
    port     = number
    username = string
    password = string
  })
  description = "Vars for event-source database"
}

variable "plugin_registry_bucket_aws_account_id" {
  type        = string
  description = "The account id that owns the bucket where plugins are stored"
}

variable "plugin_registry_bucket_name" {
  type        = string
  description = "The name of the bucket where plugins are stored"
}

variable "num_graph_mergers" {
  type        = number
  default     = 1
  description = "The number of graph merger instances to run."
}

variable "test_user_name" {
  type        = string
  description = "The name of the test user"
}

variable "num_node_identifiers" {
  type        = number
  default     = 1
  description = "The number of node identifiers to run."
}

variable "user_auth_table" {
  type        = string
  description = "What is the name of the DynamoDB user auth table?"
}

variable "user_session_table" {
  type        = string
  description = "What is the name of the DynamoDB user session table?"
}

variable "google_client_id" {
  type        = string
  description = "Google client ID used for authenticating web users via Sign In With Google"
}

variable "dns_server" {
  type        = string
  description = "The network.dns.server value. This should be equivalent to the host's ip in order to communicate with dnsmasq and allow consul dns to be available from within containers. This can be replaced as of Nomad 1.3.0 with variable interpolation per https://github.com/hashicorp/nomad/issues/11851."
  default     = ""
}

locals {
  _redis_trimmed = trimprefix(var.redis_endpoint, "redis://")
  _redis         = split(":", local._redis_trimmed)
  redis_host     = local._redis[0]
  redis_port     = local._redis[1]

  # TODO once we upgrade to nomad 1.3.0 replace this with attr.unique.network.ip-address (variable interpolation is
  # added for network.dns as of 1.3.0
  dns_servers = [var.dns_server]

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

  #######################################
  ## Begin actual Grapl core services ##
  #######################################

  group "generator-dispatcher" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
    }

    task "generator-dispatcher" {
      driver = "docker"

      config {
        image = var.container_images["generator-dispatcher"]
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        # Upstreams
        PLUGIN_WORK_QUEUE_CLIENT_ADDRESS = "http://${NOMAD_UPSTREAM_ADDR_plugin-work-queue}"
        PLUGIN_REGISTRY_CLIENT_ADDRESS   = "http://${NOMAD_UPSTREAM_ADDR_plugin-registry}"

        # Kafka
        KAFKA_BOOTSTRAP_SERVERS   = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME       = var.kafka_credentials["generator-dispatcher"].sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials["generator-dispatcher"].sasl_password
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_groups["generator-dispatcher"]
        KAFKA_CONSUMER_TOPIC      = "raw-logs"
        KAFKA_PRODUCER_TOPIC      = "generated-graphs"
        KAFKA_RETRY_TOPIC         = "raw-logs-retry"

        # TODO: should equal number of raw-logs partitions
        WORKER_POOL_SIZE = 10

        GENERATOR_IDS_CACHE_CAPACITY            = 10000
        GENERATOR_IDS_CACHE_TTL_MS              = 500
        GENERATOR_IDS_CACHE_UPDATER_POOL_SIZE   = 10
        GENERATOR_IDS_CACHE_UPDATER_QUEUE_DEPTH = 1000

        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log
      }
    }

    service {
      name = "generator-dispatcher"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "plugin-work-queue"
              local_bind_port  = 1000
            }
            upstreams {
              destination_name = "plugin-registry"
              local_bind_port  = 1001
            }
          }
        }
      }
    }
  }

  group "graph-merger" {
    count = var.num_graph_mergers

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
    }

    task "graph-merger" {
      driver = "docker"

      config {
        image = var.container_images["graph-merger"]
      }

      # This writes an env file that gets read by nomad automatically
      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        AWS_REGION         = var.aws_region
        RUST_LOG           = var.rust_log
        RUST_BACKTRACE     = local.rust_backtrace
        REDIS_ENDPOINT     = var.redis_endpoint
        GRAPL_SCHEMA_TABLE = var.schema_table_name

        GRAPH_MUTATION_CLIENT_URL = "http://${NOMAD_UPSTREAM_ADDR_graph-mutation-service}"

        KAFKA_BOOTSTRAP_SERVERS   = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME       = var.kafka_credentials["graph-merger"].sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials["graph-merger"].sasl_password
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_groups["graph-merger"]
        KAFKA_CONSUMER_TOPIC      = "identified-graphs"
        KAFKA_PRODUCER_TOPIC      = "graph-updates"

      }
    }

    service {
      name = "graph-merger"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "graph-mutation-service"
              local_bind_port  = 9998
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
      dns {
        servers = local.dns_servers
      }
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

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        AWS_REGION = var.aws_region

        RUST_LOG       = var.rust_log
        RUST_BACKTRACE = local.rust_backtrace

        UID_ALLOCATOR_URL         = "http://${NOMAD_UPSTREAM_ADDR_uid-allocator}"
        KAFKA_BOOTSTRAP_SERVERS   = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME       = var.kafka_credentials["node-identifier"].sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials["node-identifier"].sasl_password
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_groups["node-identifier"]
        KAFKA_CONSUMER_TOPIC      = "generated-graphs"
        KAFKA_PRODUCER_TOPIC      = "identified-graphs"

        GRAPL_SCHEMA_TABLE          = var.schema_table_name
        GRAPL_DYNAMIC_SESSION_TABLE = var.session_table_name
        GRAPL_STATIC_MAPPING_TABLE  = var.static_mapping_table_name
      }
    }

    service {
      name = "node-identifier"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "uid-allocator"
              local_bind_port  = 9998
            }
          }
        }
      }
    }
  }

  group "engagement-creator" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
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

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        AWS_DEFAULT_REGION = var.aws_region

        GRAPL_LOG_LEVEL = var.py_log_level

        # todo: Add the mutation service
        SOURCE_QUEUE_URL = "fake"
      }
    }


    service {
      name = "engagement-creator"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "graph-mutation-service"
              local_bind_port  = 1000
            }
          }
        }
      }
    }
  }

  group "graphql-endpoint" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
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

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        RUST_LOG = var.rust_log
        # JS SDK only recognized AWS_REGION whereas rust and python SDKs use DEFAULT_AWS_REGION
        AWS_REGION = var.aws_region
        # Add the graph mutation service
        GRAPL_SCHEMA_TABLE            = var.schema_table_name
        GRAPL_SCHEMA_PROPERTIES_TABLE = var.schema_properties_table_name
        IS_LOCAL                      = "True"
        JWT_SECRET_ID                 = "JWT_SECRET_ID"
        PORT                          = "${NOMAD_PORT_graphql-endpoint-port}"
      }
    }

    service {
      name = "graphql-endpoint"
      port = "graphql-endpoint-port"

      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "graph-mutation-service"
              local_bind_port  = 1000
            }
          }
        }
      }
    }
  }

  group "kafka-retry" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
    }

    task "generator-dispatcher-retry" {
      driver = "docker"

      config {
        image = var.container_images["kafka-retry"]
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        # Kafka
        KAFKA_BOOTSTRAP_SERVERS   = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME       = var.kafka_credentials["generator-dispatcher-retry"].sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials["generator-dispatcher-retry"].sasl_password
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_groups["generator-dispatcher-retry"]
        KAFKA_RETRY_TOPIC         = "raw-logs-retry"
        KAFKA_PRODUCER_TOPIC      = "raw-logs"

        # TODO: should equal number of raw-logs-retry partitions
        KAFKA_RETRY_WORKER_POOL_SIZE = 10

        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log
      }

      service {
        name = "generator-dispatcher-retry"
      }
    }
  }

  group "web-ui" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }

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

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
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
        GRAPL_GOOGLE_CLIENT_ID               = var.google_client_id
        RUST_LOG                             = var.rust_log
        RUST_BACKTRACE                       = local.rust_backtrace
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
      dns {
        servers = local.dns_servers
      }
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

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        AWS_REGION = var.aws_region

        RUST_LOG       = var.rust_log
        RUST_BACKTRACE = local.rust_backtrace

        KAFKA_BOOTSTRAP_SERVERS   = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME       = var.kafka_credentials["graph-generator"].sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials["graph-generator"].sasl_password
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_groups["graph-generator"]

        # Temp, until we change sysmon-generator to use the real Plugin SDK
        KAFKA_CONSUMER_TOPIC = "raw-logs"
        KAFKA_PRODUCER_TOPIC = "generated-graphs"
      }
    }
  }

  group "organization-management" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
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

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        AWS_REGION                           = var.aws_region
        NOMAD_SERVICE_ADDRESS                = "${attr.unique.network.ip-address}:4646"
        ORGANIZATION_MANAGEMENT_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_organization-management-port}"
        RUST_BACKTRACE                       = local.rust_backtrace
        RUST_LOG                             = var.rust_log
        ORGANIZATION_MANAGEMENT_DB_HOSTNAME  = var.organization_management_db.hostname
        ORGANIZATION_MANAGEMENT_DB_PASSWORD  = var.organization_management_db.password
        ORGANIZATION_MANAGEMENT_DB_PORT      = var.organization_management_db.port
        ORGANIZATION_MANAGEMENT_DB_USERNAME  = var.organization_management_db.username

        ORGANIZATION_MANAGEMENT_HEALTHCHECK_POLLING_INTERVAL_MS = var.organization_management_healthcheck_polling_interval_ms
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

  group "pipeline-ingress" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "pipeline-ingress-port" {
      }
    }

    task "pipeline-ingress" {
      driver = "docker"

      config {
        image = var.container_images["pipeline-ingress"]
        ports = ["pipeline-ingress-port"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "pipeline-ingress-env"
        env         = true
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        AWS_REGION                                       = var.aws_region
        NOMAD_SERVICE_ADDRESS                            = "${attr.unique.network.ip-address}:4646"
        PIPELINE_INGRESS_BIND_ADDRESS                    = "0.0.0.0:${NOMAD_PORT_pipeline-ingress-port}"
        RUST_BACKTRACE                                   = local.rust_backtrace
        RUST_LOG                                         = var.rust_log
        PIPELINE_INGRESS_HEALTHCHECK_POLLING_INTERVAL_MS = var.pipeline_ingress_healthcheck_polling_interval_ms
        KAFKA_BOOTSTRAP_SERVERS                          = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME                              = var.kafka_credentials["pipeline-ingress"].sasl_username
        KAFKA_SASL_PASSWORD                              = var.kafka_credentials["pipeline-ingress"].sasl_password
        KAFKA_PRODUCER_TOPIC                             = "raw-logs"
      }
    }

    service {
      name = "pipeline-ingress"
      port = "pipeline-ingress-port"
      connect {
        sidecar_service {
        }
      }

      check {
        type     = "grpc"
        port     = "pipeline-ingress-port"
        interval = "10s"
        timeout  = "3s"
      }
    }
  }

  group "plugin-registry" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }

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

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        AWS_REGION                                      = var.aws_region
        NOMAD_SERVICE_ADDRESS                           = "${attr.unique.network.ip-address}:4646"
        PLUGIN_REGISTRY_BIND_ADDRESS                    = "0.0.0.0:${NOMAD_PORT_plugin-registry-port}"
        PLUGIN_REGISTRY_DB_HOSTNAME                     = var.plugin_registry_db.hostname
        PLUGIN_REGISTRY_DB_PASSWORD                     = var.plugin_registry_db.password
        PLUGIN_REGISTRY_DB_PORT                         = var.plugin_registry_db.port
        PLUGIN_REGISTRY_DB_USERNAME                     = var.plugin_registry_db.username
        PLUGIN_BOOTSTRAP_CONTAINER_IMAGE                = var.container_images["plugin-bootstrap"]
        PLUGIN_REGISTRY_KERNEL_ARTIFACT_URL             = var.plugin_registry_kernel_artifact_url
        PLUGIN_REGISTRY_ROOTFS_ARTIFACT_URL             = var.plugin_registry_rootfs_artifact_url
        PLUGIN_REGISTRY_HAX_DOCKER_PLUGIN_RUNTIME_IMAGE = var.container_images["hax-docker-plugin-runtime"]
        PLUGIN_EXECUTION_IMAGE                          = var.container_images["generator-execution-sidecar"] # TODO: add support for analyzer too
        PLUGIN_REGISTRY_BUCKET_AWS_ACCOUNT_ID           = var.plugin_registry_bucket_aws_account_id
        PLUGIN_REGISTRY_BUCKET_NAME                     = var.plugin_registry_bucket_name
        PLUGIN_EXECUTION_OBSERVABILITY_ENV_VARS         = var.observability_env_vars

        # common Rust env vars
        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log
      }

      resources {
        # Probably too much. Let's figure out buffered writes to s3
        memory = 512
      }
    }

    service {
      name = "plugin-registry"
      port = "plugin-registry-port"
      connect {
        sidecar_service {
        }
      }

      check {
        type     = "grpc"
        port     = "plugin-registry-port"
        interval = "10s"
        timeout  = "3s"
      }
    }
  }

  group "plugin-work-queue" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }

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

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        PLUGIN_WORK_QUEUE_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_plugin-work-queue-port}"
        PLUGIN_WORK_QUEUE_DB_HOSTNAME  = var.plugin_work_queue_db.hostname
        PLUGIN_WORK_QUEUE_DB_PASSWORD  = var.plugin_work_queue_db.password
        PLUGIN_WORK_QUEUE_DB_PORT      = var.plugin_work_queue_db.port
        PLUGIN_WORK_QUEUE_DB_USERNAME  = var.plugin_work_queue_db.username
        # Hardcoded, but makes little sense to pipe up through Pulumi
        PLUGIN_WORK_QUEUE_HEALTHCHECK_POLLING_INTERVAL_MS = 5000

        KAFKA_BOOTSTRAP_SERVERS = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME     = var.kafka_credentials["plugin-work-queue"].sasl_username
        KAFKA_SASL_PASSWORD     = var.kafka_credentials["plugin-work-queue"].sasl_password

        GENERATOR_KAFKA_PRODUCER_TOPIC = "generated-graphs"
        # ANALYZER_KAFKA_PRODUCER_TOPIC = "TODO"

        # common Rust env vars
        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log
      }
    }

    service {
      name = "plugin-work-queue"
      port = "plugin-work-queue-port"
      connect {
        sidecar_service {
        }
      }

      check {
        type     = "grpc"
        port     = "plugin-work-queue-port"
        interval = "10s"
        timeout  = "3s"
      }
    }
  }

  group "uid-allocator" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }

      port "uid-allocator-port" {
      }
    }

    task "uid-allocator" {
      driver = "docker"

      config {
        image = var.container_images["uid-allocator"]
        ports = ["uid-allocator-port"]
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        DEFAULT_ALLOCATION_SIZE    = var.uid_allocator_service_config.default_allocation_size
        PREALLOCATION_SIZE         = var.uid_allocator_service_config.preallocation_size
        MAXIMUM_ALLOCATION_SIZE    = var.uid_allocator_service_config.maximum_allocation_size
        UID_ALLOCATOR_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_uid-allocator-port}"
        UID_ALLOCATOR_DB_HOSTNAME  = var.uid_allocator_db.hostname
        UID_ALLOCATOR_DB_PASSWORD  = var.uid_allocator_db.password
        UID_ALLOCATOR_DB_PORT      = var.uid_allocator_db.port
        UID_ALLOCATOR_DB_USERNAME  = var.uid_allocator_db.username
        RUST_BACKTRACE             = local.rust_backtrace
        RUST_LOG                   = var.rust_log
      }
    }

    service {
      name = "uid-allocator"
      port = "uid-allocator-port"
      connect {
        sidecar_service {
        }
      }
    }
  }

  group "graph-mutation-service" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "graph-mutation-service-port" {
      }
    }

    task "graph-mutation-service" {
      driver = "docker"

      config {
        image = var.container_images["graph-mutation-service"]
        ports = ["graph-mutation-service-port"]
      }

      env {
        GRAPH_MUTATION_SERVICE_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_graph-mutation-service-port}"
        RUST_BACKTRACE                      = local.rust_backtrace
        RUST_LOG                            = var.rust_log
        GRAPH_DB_ADDRESSES                  = var.graph_db.addresses
        GRAPH_DB_AUTH_PASSWORD              = var.graph_db.password
        GRAPH_DB_AUTH_USERNAME              = var.graph_db.username
        SCHEMA_MANAGER_ADDRESS              = "http://${NOMAD_UPSTREAM_ADDR_schema-manager}"
        UID_ALLOCATOR_ADDRESS               = "http://${NOMAD_UPSTREAM_ADDR_uid-allocator}"
      }
    }

    service {
      name = "graph-mutation-service"
      port = "graph-mutation-service-port"
      connect {
        sidecar_service {
          proxy {
            config {
              protocol = "grpc"
            }
            # It'd be nice to dynamically use ports. Sadly, per https://github.com/hashicorp/nomad/issues/7135 its not
            # to be. The ports chosen below can be changed at any time
            upstreams {
              destination_name = "schema-manager"
              local_bind_port  = 9999
            }

            upstreams {
              destination_name = "uid-allocator"
              local_bind_port  = 9998
            }
          }
        }
      }
    }
  }


  group "schema-manager" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "schema-manager-port" {
      }
    }

    task "schema-manager" {
      driver = "docker"

      config {
        image = var.container_images["schema-manager"]
        ports = ["schema-manager-port"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "schema-manager.env"
        env         = true
      }

      env {
        NOMAD_SERVICE_ADDRESS       = "${attr.unique.network.ip-address}:4646"
        SCHEMA_SERVICE_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_schema-manager-port}"
        RUST_BACKTRACE              = local.rust_backtrace
        RUST_LOG                    = var.rust_log
        SCHEMA_DB_HOSTNAME          = var.schema_manager_db.hostname
        SCHEMA_DB_PASSWORD          = var.schema_manager_db.password
        SCHEMA_DB_PORT              = var.schema_manager_db.port
        SCHEMA_DB_USERNAME          = var.schema_manager_db.username
      }
    }

    service {
      name = "schema-manager"
      port = "schema-manager-port"
      connect {
        sidecar_service {
        }
      }
    }
  }


  group "event-source" {
    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }

      port "event-source-port" {
      }
    }

    task "event-source" {
      driver = "docker"

      config {
        image = var.container_images["event-source"]
        ports = ["event-source-port"]
      }

      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      template {
        data        = var.observability_env_vars
        destination = "observability.env"
        env         = true
      }

      env {
        EVENT_SOURCE_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_event-source-port}"
        EVENT_SOURCE_DB_HOSTNAME  = var.event_source_db.hostname
        EVENT_SOURCE_DB_PASSWORD  = var.event_source_db.password
        EVENT_SOURCE_DB_PORT      = var.event_source_db.port
        EVENT_SOURCE_DB_USERNAME  = var.event_source_db.username
        # Hardcoded, but makes little sense to pipe up through Pulumi
        EVENT_SOURCE_HEALTHCHECK_POLLING_INTERVAL_MS = 5000

        # common Rust env vars
        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log
      }
    }

    service {
      name = "event-source"
      port = "event-source-port"
      connect {
        sidecar_service {
        }
      }

      check {
        type     = "grpc"
        port     = "event-source-port"
        interval = "10s"
        timeout  = "3s"
      }
    }
  }
}
