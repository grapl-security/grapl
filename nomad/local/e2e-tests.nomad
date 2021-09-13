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

variable "analyzer_bucket" {
  type        = string
  description = "The s3 bucket which the analyzer stores items to analyze"
}

variable "sysmon_generator_queue" {
  type        = string
  description = "The URL of the SQS queue for Sysmon logs"
}

variable "sysmon_log_bucket" {
  type        = string
  description = "The name of the S3 bucket to which Sysmon logs should be uploaded"
}

variable "schema_table_name" {
  type        = string
  description = "What is the name of the schema table?"
}

variable "schema_properties_table_name" {
  type        = string
  description = "What is the name of the schema properties table?"
}

variable "aws_access_key_id" {
  type        = string
  description = "The aws access key id used to interact with AWS."
}

variable "aws_access_key_secret" {
  type        = string
  description = "The aws access key secret used to interact with AWS."
}

variable "_aws_endpoint" {
  type        = string
  description = "The endpoint in which we can expect to find and interact with AWS."
}

variable "test_user_name" {
  type        = string
  description = "The name of the test user"
}

locals {
  log_level = "DEBUG"

  # Prefer these over their `var` equivalents
  aws_endpoint = replace(var._aws_endpoint, "LOCAL_GRAPL_REPLACE_IP", attr.unique.network.ip-address)
}

job "e2e-tests" {
  datacenters = ["dc1"]
  type        = "batch"
  parameterized {}

  reschedule {
    # Make this a one-shot job
    attempts = 0
  }

  # Specifies that this job is the most high priority job we have; nothing else should take precedence 
  priority = 100

  group "e2e-tests" {
    restart {
      # Make this a one-shot job
      attempts = 0
    }

    network {
      mode = "bridge"
    }

    # Enable service discovery
    service {
      name = "e2e-tests"
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

    task "e2e-tests-setup" {
      driver = "docker"

      config {
        image      = "${var.container_registry}grapl/e2e-tests:dev"
        entrypoint = ["/bin/bash", "-o", "errexit", "-o", "nounset", "-c"]
        command = trimspace(<<EOF
graplctl upload analyzer --analyzer_main_py ./etc/local_grapl/suspicious_svchost/main.py
graplctl upload analyzer --analyzer_main_py ./etc/local_grapl/unique_cmd_parent/main.py
graplctl upload sysmon --logfile ./etc/sample_data/eventlog.xml
EOF
        )
      }

      env {
        GRAPL_REGION    = var.aws_region
        DEPLOYMENT_NAME = var.deployment_name

        GRAPL_AWS_ENDPOINT          = local.aws_endpoint
        GRAPL_AWS_ACCESS_KEY_ID     = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET = var.aws_access_key_secret

        GRAPL_ANALYZERS_BUCKET       = var.analyzer_bucket
        GRAPL_SYSMON_GENERATOR_QUEUE = var.sysmon_generator_queue
        GRAPL_SYSMON_LOG_BUCKET      = var.sysmon_log_bucket

        # These are needed due to graplctl's idempotency checks
        GRAPL_SCHEMA_TABLE            = var.schema_table_name
        GRAPL_SCHEMA_PROPERTIES_TABLE = var.schema_properties_table_name

        # TODO: I'm not sure why we need GRAPL_VERSION=
        GRAPL_VERSION = var.deployment_name
      }

      # Run `e2e-tests-setup` before `e2e-tests`
      lifecycle {
        hook    = "prestart"
        sidecar = false
      }
    }

    task "e2e-tests" {
      driver = "docker"

      config {
        image      = "${var.container_registry}grapl/e2e-tests:dev"
        entrypoint = ["/bin/bash", "-o", "errexit", "-o", "nounset", "-c"]
        command    = "python3 -c 'import lambdex_handler; lambdex_handler.handler(None, None)'"
      }

      env {
        AWS_REGION = var.aws_region
        # TODO: Reintroduce DEBUG_SERVICES= at some point
        # TODO: Reintroduce VSC_DEBUGGER_PORT= at some point
        # TODO: GRAPL_API_HOST - we need this for JWTs
        # TODO: GRAPL_HTTP_FRONTEND_PORT - need this for JWTs
        DEPLOYMENT_NAME             = var.deployment_name
        GRAPL_AWS_ENDPOINT          = local.aws_endpoint
        GRAPL_AWS_ACCESS_KEY_ID     = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET = var.aws_access_key_secret
        GRAPL_LOG_LEVEL             = local.log_level

        GRAPL_TEST_USER_NAME         = var.test_user_name  # Needed for EngagementEdgeClient
        IS_LOCAL                     = true # Revisit for in-prod E2E

        MG_ALPHAS      = "localhost:9080"
        RUST_BACKTRACE = 1
        RUST_LOG       = local.log_level

      }
    }
  }
}