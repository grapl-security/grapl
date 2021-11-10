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
  default     = "DUMMY_LOCAL_AWS_ACCESS_KEY_ID"
}

variable "aws_access_key_secret" {
  type        = string
  description = "The aws access key secret used to interact with AWS."
  default     = "DUMMY_LOCAL_AWS_ACCESS_KEY_SECRET"
}

variable "_aws_endpoint" {
  type        = string
  description = "The endpoint in which we can expect to find and interact with AWS."
  default     = "DUMMY_LOCAL_AWS_ENDPOINT"
}

variable "test_user_name" {
  type        = string
  description = "The name of the test user"
}

locals {
  log_level = "DEBUG"

  # Prefer these over their `var` equivalents
  aws_endpoint = replace(var._aws_endpoint, "LOCAL_GRAPL_REPLACE_IP", "{{ env \"attr.unique.network.ip-address\" }}")

  # This is used to conditionally submit env variables via template stanzas.
  local_only_env_vars = <<EOH
GRAPL_AWS_ENDPOINT          = ${local.aws_endpoint}
GRAPL_AWS_ACCESS_KEY_ID     = ${var.aws_access_key_id}
GRAPL_AWS_ACCESS_KEY_SECRET = ${var.aws_access_key_secret}
EOH
  # We need to submit an env var otherwise you can end up with a weird nomad state parse error.
  aws_only_env_vars              = "DUMMY_VAR=TRUE"
  conditionally_defined_env_vars = (var._aws_endpoint == "http://LOCAL_GRAPL_REPLACE_IP:4566") ? local.local_only_env_vars : local.aws_only_env_vars
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
      # TODO: Reintroduce VSC_DEBUGGER_PORT_FOR_GRAPL_E2E_TESTS at some point
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
            upstreams {
              destination_name = "web-ui"
              local_bind_port  = 1234
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

      # This writes an env file that gets read by the task automatically
      template {
        data        = local.conditionally_defined_env_vars
        destination = "e2e-tests-setup.env"
        env         = true
      }

      env {
        GRAPL_REGION    = var.aws_region
        DEPLOYMENT_NAME = var.deployment_name

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
        image = "${var.container_registry}grapl/e2e-tests:dev"
      }

      # This writes an env file that gets read by the task automatically
      template {
        data        = local.conditionally_defined_env_vars
        destination = "e2e-tests.env"
        env         = true
      }

      env {
        AWS_REGION = var.aws_region
        # TODO: Reintroduce DEBUG_SERVICES= at some point
        # TODO: Reintroduce VSC_DEBUGGER_PORT= at some point

        GRAPL_API_HOST           = "${NOMAD_UPSTREAM_IP_web-ui}"
        GRAPL_HTTP_FRONTEND_PORT = "${NOMAD_UPSTREAM_PORT_web-ui}"

        DEPLOYMENT_NAME = var.deployment_name
        GRAPL_LOG_LEVEL = local.log_level

        GRAPL_TEST_USER_NAME = var.test_user_name # Needed for EngagementEdgeClient
        IS_LOCAL             = true               # Revisit for in-prod E2E

        MG_ALPHAS      = "localhost:9080"
        RUST_BACKTRACE = 1
        RUST_LOG       = local.log_level

      }
    }
  }
}