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

variable "kafka_bootstrap_servers" {
  type        = string
  description = "The URL(s) (possibly comma-separated) of the Kafka bootstrap servers."
}

variable "pipeline_ingress_healthcheck_polling_interval_ms" {
  type        = string
  description = "The amount of time to wait between each healthcheck execution."
}

variable "pipeline_ingress_kafka_consumer_group_name" {
  type        = string
  description = "The name of the consumer group the pipeline-ingress consumers will join."
}

variable "pipeline_ingress_kafka_sasl_username" {
  type        = string
  description = "The Confluent Cloud API key to configure pipeline-ingress producers and consumers with."
}

variable "pipeline_ingress_kafka_sasl_password" {
  type        = string
  description = "The Confluent Cloud API secret to configure pipeline-ingress producers and consumers with."
}

locals {
  log_level = "DEBUG"
}

job "integration-tests-new" {
  datacenters = ["dc1"]
  type        = "batch"
  parameterized {}

  reschedule {
    # Make this a one-shot job
    attempts = 0
  }

  # Specifies that this job is the most high priority job we have; nothing else should take precedence
  priority = 100

  group "integration-tests-new" {
    restart {
      # Make this a one-shot job
      attempts = 0
    }

    network {
      mode = "bridge"
    }

    # Enable service discovery
    service {
      name = "integration-tests-new"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "pipeline-ingress"
              # port unique but arbitrary - https://github.com/hashicorp/nomad/issues/7135
              local_bind_port = 1000
            }
          }
        }
      }
    }

    task "rust-integration-tests-new" {
      driver = "docker"

      config {
        image = var.container_images["rust-integration-tests-new"]
      }

      # This writes an env file that gets read by the task automatically
      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      env {
        AWS_REGION = var.aws_region

        GRAPL_LOG_LEVEL = local.log_level

        RUST_BACKTRACE = 1
        RUST_LOG       = local.log_level

        KAFKA_BOOTSTRAP_SERVERS = var.kafka_bootstrap_servers

        PIPELINE_INGRESS_CLIENT_ADDRESS                 = "http://${NOMAD_UPSTREAM_ADDR_pipeline_ingress}"
        PIPELINE_INGRESS_KAFKA_SASL_USERNAME            = var.pipeline_ingress_kafka_sasl_username
        PIPELINE_INGRESS_KAFKA_SASL_PASSWORD            = var.pipeline_ingress_kafka_sasl_password
        PIPELINE_INGRESS_TEST_KAFKA_CONSUMER_GROUP_NAME = "pipeline-ingress-test"

        NOMAD_SERVICE_ADDRESS = "${attr.unique.network.ip-address}:4646"
      }
    }
  }
}
