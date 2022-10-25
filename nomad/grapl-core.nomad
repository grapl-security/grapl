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

variable "static_mapping_table_name" {
  type        = string
  description = "The name of the dynamodb table used for storing the ids of static nodes"
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

variable "test_user_name" {
  type        = string
  description = "The name of the test user"
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

locals {
  dns_servers = [attr.unique.network.ip-address]

  # enabled
  rust_backtrace = 1

  # Set up default tags for otel traces via the OTEL_RESOURCE_ATTRIBUTES env variable. Format is key=value,key=value
  # We're setting up defaults on a per-job basis, but these can be expanded on a per-service basis as necessary.
  # Examples of keys we may add in the future: language, instance_id/ip, team

  # Currently we use the same version for all containers. As such we pick one container to get the version from
  app_version                      = split(":", var.container_images["analyzer-dispatcher"])[1]
  default_otel_resource_attributes = "service.version=${local.app_version},host.hostname=${attr.unique.hostname}"
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

  group "analyzer-dispatcher" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "metrics_envoy" { to = 9102 }
    }

    task "analyzer-dispatcher" {
      driver = "docker"

      config {
        image = var.container_images["analyzer-dispatcher"]
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
        KAFKA_SASL_USERNAME       = var.kafka_credentials["analyzer-dispatcher"].sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials["analyzer-dispatcher"].sasl_password
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_groups["analyzer-dispatcher"]
        KAFKA_CONSUMER_TOPIC      = "merged-graphs"
        KAFKA_RETRY_TOPIC         = "merged-graphs-retry"

        # should equal number of merged-graphs partitions
        WORKER_POOL_SIZE = 2

        ANALYZER_IDS_CACHE_CAPACITY            = 10000
        ANALYZER_IDS_CACHE_TTL_MS              = 5000
        ANALYZER_IDS_CACHE_UPDATER_POOL_SIZE   = 10
        ANALYZER_IDS_CACHE_UPDATER_QUEUE_DEPTH = 1000

        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log

        OTEL_RESOURCE_ATTRIBUTES = local.default_otel_resource_attributes
      }

      resources {
        cpu = 50
      }
    }

    service {
      meta {
        # Tag for prometheus scrape-targeting via consul (envoy)
        metrics_port_envoy = "${NOMAD_HOST_PORT_metrics_envoy}"
      }
      name = "analyzer-dispatcher"
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

  group "generator-dispatcher" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "metrics_envoy" { to = 9102 }
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
        KAFKA_RETRY_TOPIC         = "raw-logs-retry"

        # should equal number of raw-logs partitions
        WORKER_POOL_SIZE = 2

        GENERATOR_IDS_CACHE_CAPACITY            = 10000
        GENERATOR_IDS_CACHE_TTL_MS              = 5000
        GENERATOR_IDS_CACHE_UPDATER_POOL_SIZE   = 10
        GENERATOR_IDS_CACHE_UPDATER_QUEUE_DEPTH = 1000

        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log

        OTEL_RESOURCE_ATTRIBUTES = local.default_otel_resource_attributes
      }

      resources {
        cpu = 50
      }
    }

    service {
      meta {
        # Tag for prometheus scrape-targeting via consul (envoy)
        metrics_port_envoy = "${NOMAD_HOST_PORT_metrics_envoy}"
      }
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
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "metrics_envoy" { to = 9102 }
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
        AWS_REGION     = var.aws_region
        RUST_LOG       = var.rust_log
        RUST_BACKTRACE = local.rust_backtrace

        GRAPH_MUTATION_CLIENT_ADDRESS = "http://${NOMAD_UPSTREAM_ADDR_graph-mutation}"

        GRAPL_SCHEMA_TABLE = var.schema_table_name

        KAFKA_BOOTSTRAP_SERVERS   = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME       = var.kafka_credentials["graph-merger"].sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials["graph-merger"].sasl_password
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_groups["graph-merger"]
        KAFKA_CONSUMER_TOPIC      = "identified-graphs"
        KAFKA_PRODUCER_TOPIC      = "merged-graphs"

        OTEL_RESOURCE_ATTRIBUTES = local.default_otel_resource_attributes
      }

      resources {
        cpu = 50
      }
    }

    service {
      meta {
        # Tag for prometheus scrape-targeting via consul (envoy)
        metrics_port_envoy = "${NOMAD_HOST_PORT_metrics_envoy}"
      }
      name = "graph-merger"

      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "graph-mutation"
              local_bind_port  = 1001
            }
          }
        }
      }
    }
  }

  group "node-identifier" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "metrics_envoy" { to = 9102 }
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

        KAFKA_BOOTSTRAP_SERVERS   = var.kafka_bootstrap_servers
        KAFKA_SASL_USERNAME       = var.kafka_credentials["node-identifier"].sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials["node-identifier"].sasl_password
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_groups["node-identifier"]
        KAFKA_CONSUMER_TOPIC      = "generated-graphs"
        KAFKA_PRODUCER_TOPIC      = "identified-graphs"

        GRAPH_MUTATION_CLIENT_ADDRESS = "http://${NOMAD_UPSTREAM_ADDR_graph-mutation}"

        GRAPL_SCHEMA_TABLE          = var.schema_table_name
        GRAPL_DYNAMIC_SESSION_TABLE = var.session_table_name
        GRAPL_STATIC_MAPPING_TABLE  = var.static_mapping_table_name

        OTEL_RESOURCE_ATTRIBUTES = local.default_otel_resource_attributes
      }

      resources {
        cpu = 50
      }
    }
    service {
      meta {
        # Tag for prometheus scrape-targeting via consul (envoy)
        metrics_port_envoy = "${NOMAD_HOST_PORT_metrics_envoy}"
      }
      name = "node-identifier"

      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "graph-mutation"
              local_bind_port  = 1001
            }
          }
        }
      }
    }
  }

  group "kafka-retry" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "metrics_envoy" { to = 9102 }
    }

    task "analyzer-dispatcher-retry" {
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
        KAFKA_SASL_USERNAME       = var.kafka_credentials["analyzer-dispatcher-retry"].sasl_username
        KAFKA_SASL_PASSWORD       = var.kafka_credentials["analyzer-dispatcher-retry"].sasl_password
        KAFKA_CONSUMER_GROUP_NAME = var.kafka_consumer_groups["analyzer-dispatcher-retry"]
        KAFKA_RETRY_TOPIC         = "merged-graphs-retry"
        KAFKA_RETRY_DELAY_MS      = 500
        KAFKA_PRODUCER_TOPIC      = "merged-graphs"

        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log

        OTEL_RESOURCE_ATTRIBUTES = local.default_otel_resource_attributes
      }

      resources {
        cpu = 50
      }

      service {
        meta {
          # Tag for prometheus scrape-targeting via consul (envoy)
          metrics_port_envoy = "${NOMAD_HOST_PORT_metrics_envoy}"
        }
        name = "analyzer-dispatcher-retry"
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
        KAFKA_RETRY_DELAY_MS      = 500
        KAFKA_PRODUCER_TOPIC      = "raw-logs"

        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log

        OTEL_RESOURCE_ATTRIBUTES = local.default_otel_resource_attributes
      }

      resources {
        cpu = 50
      }

      service {
        meta {
          # Tag for prometheus scrape-targeting via consul (envoy)
          metrics_port_envoy = "${NOMAD_HOST_PORT_metrics_envoy}"
        }
        name = "generator-dispatcher-retry"
      }
    }
  }

  group "web-ui" {
    count = 1

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "metrics_envoy" { to = 9102 }

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

        PLUGIN_REGISTRY_CLIENT_ADDRESS  = "http://${NOMAD_UPSTREAM_ADDR_plugin-registry}"
        PIPELINE_INGRESS_CLIENT_ADDRESS = "http://${NOMAD_UPSTREAM_ADDR_pipeline-ingress}"

        GRAPL_WEB_UI_BIND_ADDRESS = "0.0.0.0:${NOMAD_PORT_web-ui-port}"
        GRAPL_GOOGLE_CLIENT_ID    = var.google_client_id
        RUST_LOG                  = var.rust_log
        RUST_BACKTRACE            = local.rust_backtrace

        OTEL_RESOURCE_ATTRIBUTES = local.default_otel_resource_attributes
      }

      resources {
        # Can OOM with default limit of 300 MB and a number of connections at the same
        # time, which seems low, but is reliably produced with the its integration tests.
        memory_max = 1024
        cpu        = 50
      }
    }

    service {
      meta {
        # Tag for prometheus scrape-targeting via consul (envoy)
        metrics_port_envoy = "${NOMAD_HOST_PORT_metrics_envoy}"
      }
      name = "web-ui"
      port = "web-ui-port"
      connect {
        sidecar_service {
          proxy {
            config {
              protocol = "http"
            }
            upstreams {
              destination_name = "plugin-registry"
              local_bind_port  = 1001
            }
            upstreams {
              destination_name = "pipeline-ingress"
              local_bind_port  = 1002
            }
          }
        }
      }
      check {
        type     = "http"
        name     = "grapl-web-health"
        path     = "/api/health"
        interval = "10s"
        timeout  = "2s"
      }
    }
  }

  group "organization-management" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "metrics_envoy" { to = 9102 }
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
        ORGANIZATION_MANAGEMENT_DB_ADDRESS   = "${var.organization_management_db.hostname}:${var.organization_management_db.port}"
        ORGANIZATION_MANAGEMENT_DB_PASSWORD  = var.organization_management_db.password
        ORGANIZATION_MANAGEMENT_DB_USERNAME  = var.organization_management_db.username

        ORGANIZATION_MANAGEMENT_HEALTHCHECK_POLLING_INTERVAL_MS = var.organization_management_healthcheck_polling_interval_ms

        OTEL_RESOURCE_ATTRIBUTES = local.default_otel_resource_attributes
      }

      resources {
        cpu = 50
      }
    }

    service {
      meta {
        # Tag for prometheus scrape-targeting via consul (envoy)
        metrics_port_envoy = "${NOMAD_HOST_PORT_metrics_envoy}"
      }
      name = "organization-management"
      port = "organization-management-port"
      connect {
        sidecar_service {
        }
      }
    }
  }

  group "pipeline-ingress" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "metrics_envoy" { to = 9102 }
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

        OTEL_RESOURCE_ATTRIBUTES = local.default_otel_resource_attributes
      }

      resources {
        cpu = 50
      }
    }

    service {
      meta {
        # Tag for prometheus scrape-targeting via consul (envoy)
        metrics_port_envoy = "${NOMAD_HOST_PORT_metrics_envoy}"
      }
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
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "metrics_envoy" { to = 9102 }

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
        PLUGIN_REGISTRY_DB_ADDRESS                      = "${var.plugin_registry_db.hostname}:${var.plugin_registry_db.port}"
        PLUGIN_REGISTRY_DB_PASSWORD                     = var.plugin_registry_db.password
        PLUGIN_REGISTRY_DB_USERNAME                     = var.plugin_registry_db.username
        PLUGIN_BOOTSTRAP_CONTAINER_IMAGE                = var.container_images["plugin-bootstrap"]
        PLUGIN_REGISTRY_KERNEL_ARTIFACT_URL             = var.plugin_registry_kernel_artifact_url
        PLUGIN_REGISTRY_ROOTFS_ARTIFACT_URL             = var.plugin_registry_rootfs_artifact_url
        PLUGIN_REGISTRY_HAX_DOCKER_PLUGIN_RUNTIME_IMAGE = var.container_images["hax-docker-plugin-runtime"]
        PLUGIN_REGISTRY_BUCKET_AWS_ACCOUNT_ID           = var.plugin_registry_bucket_aws_account_id
        PLUGIN_REGISTRY_BUCKET_NAME                     = var.plugin_registry_bucket_name
        PLUGIN_EXECUTION_GENERATOR_SIDECAR_IMAGE        = var.container_images["generator-execution-sidecar"]
        PLUGIN_EXECUTION_ANALYZER_SIDECAR_IMAGE         = var.container_images["analyzer-execution-sidecar"]
        PLUGIN_EXECUTION_GRAPH_QUERY_PROXY_IMAGE        = var.container_images["graph-query-proxy"]

        # common Rust env vars
        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log

        OTEL_RESOURCE_ATTRIBUTES = local.default_otel_resource_attributes
      }

      resources {
        # Probably too much. Let's figure out buffered writes to s3
        memory_max = 512
        cpu        = 50
      }
    }

    service {
      meta {
        # Tag for prometheus scrape-targeting via consul (envoy)
        metrics_port_envoy = "${NOMAD_HOST_PORT_metrics_envoy}"
      }
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
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "metrics_envoy" { to = 9102 }

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
        PLUGIN_WORK_QUEUE_DB_ADDRESS   = "${var.plugin_work_queue_db.hostname}:${var.plugin_work_queue_db.port}"
        PLUGIN_WORK_QUEUE_DB_PASSWORD  = var.plugin_work_queue_db.password
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

        OTEL_RESOURCE_ATTRIBUTES = local.default_otel_resource_attributes
      }

      resources {
        cpu = 50
      }
    }

    service {
      meta {
        # Tag for prometheus scrape-targeting via consul (envoy)
        metrics_port_envoy = "${NOMAD_HOST_PORT_metrics_envoy}"
      }
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

  group "event-source" {
    count = 2

    network {
      mode = "bridge"
      dns {
        servers = local.dns_servers
      }
      port "metrics_envoy" { to = 9102 }

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
        EVENT_SOURCE_DB_ADDRESS   = "${var.event_source_db.hostname}:${var.event_source_db.port}"
        EVENT_SOURCE_DB_PASSWORD  = var.event_source_db.password
        EVENT_SOURCE_DB_USERNAME  = var.event_source_db.username
        # Hardcoded, but makes little sense to pipe up through Pulumi
        EVENT_SOURCE_HEALTHCHECK_POLLING_INTERVAL_MS = 5000

        # common Rust env vars
        RUST_BACKTRACE = local.rust_backtrace
        RUST_LOG       = var.rust_log

        OTEL_RESOURCE_ATTRIBUTES = local.default_otel_resource_attributes
      }

      resources {
        cpu = 50
      }
    }

    service {
      meta {
        # Tag for prometheus scrape-targeting via consul (envoy)
        metrics_port_envoy = "${NOMAD_HOST_PORT_metrics_envoy}"
      }
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
