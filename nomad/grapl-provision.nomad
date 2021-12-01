# This job is separate from `grapl-core.nomad` because
# we want to have this job depend on
# the successful, complete startup of `dgraph` in grapl-core.
# There are more-robust ways to do this, that could bring
# `grapl-provisioner` back  into the `grapl-core` fold, but
# this will get the job done for the time being.

variable "deployment_name" {
  type        = string
  description = "The deployment name"
}

variable "_aws_endpoint" {
  type        = string
  default     = "DUMMY_LOCAL_AWS_ENDPOINT"
  description = <<EOF
  The endpoint in which we can expect to find and interact with AWS. 
  It accepts a special sentinel value domain, LOCAL_GRAPL_REPLACE_IP:xxxx, if the
  user wishes to contact Localstack.

  Prefer using `local.aws_endpoint`.
EOF
}

variable "container_images" {
  type        = map(string)
  description = <<EOF
  A map of $NAME_OF_TASK to the URL for that task's docker image ID.
  (See DockerImageId in Pulumi for further documentation.)
EOF
}

variable "aws_access_key_id" {
  type        = string
  default     = "DUMMY_LOCAL_AWS_ACCESS_KEY_ID"
  description = "The aws access key id used to interact with AWS."
}

variable "aws_access_key_secret" {
  type        = string
  default     = "DUMMY_LOCAL_AWS_ACCESS_KEY_SECRET"
  description = "The aws access key secret used to interact with AWS."
}

variable "aws_region" {
  type = string
}

variable "schema_table_name" {
  type        = string
  description = "What is the name of the schema table?"
}

variable "schema_properties_table_name" {
  type        = string
  description = "What is the name of the schema properties table?"
}

variable "user_auth_table" {
  type        = string
  description = "What is the name of the DynamoDB user auth table?"
}

variable "test_user_name" {
  type        = string
  description = "The name of the test user"
}

variable "rust_log" {
  type        = string
  description = "Controls the logging behavior of Rust-based services."
}

locals {
  # Prefer these over their `var` equivalents.
  # The aws endpoint is in template env format
  aws_endpoint = replace(var._aws_endpoint, "LOCAL_GRAPL_REPLACE_IP", "{{ env \"attr.unique.network.ip-address\" }}")

  # This is used to conditionally submit env variables via template stanzas.
  local_only_env_vars = <<EOH
GRAPL_AWS_ENDPOINT          = ${local.aws_endpoint}
GRAPL_AWS_ACCESS_KEY_ID     = ${var.aws_access_key_id}
GRAPL_AWS_ACCESS_KEY_SECRET = ${var.aws_access_key_secret}
EOH
  # We need to submit an env var otherwise you can end up with a weird nomad state parse error
  aws_only_env_vars              = "DUMMY_VAR=TRUE"
  conditionally_defined_env_vars = (var._aws_endpoint == "http://LOCAL_GRAPL_REPLACE_IP:4566") ? local.local_only_env_vars : local.aws_only_env_vars
}

job "grapl-provision" {
  datacenters = ["dc1"]

  # This means "Run it once until it's successful" - perfect for provisioner!
  type = "batch"

  group "provisioner" {
    network {
      mode = "bridge"
    }

    task "provisioner" {
      driver = "docker"

      config {
        image = var.container_images["provisioner"]
      }

      lifecycle {
        hook = "poststart"
        # Ephemeral, not long-lived
        sidecar = false
      }

      # This writes an env files that gets read by nomad automatically
      template {
        data        = local.conditionally_defined_env_vars
        destination = "provisioner.env"
        env         = true
      }

      env {
        # This is a hack, because IDK how to share locals across files.
        # It's fine if `provision` only hits one alpha.
        MG_ALPHAS = "localhost:9080"

        DEPLOYMENT_NAME               = var.deployment_name
        AWS_DEFAULT_REGION            = var.aws_region
        GRAPL_SCHEMA_TABLE            = var.schema_table_name
        GRAPL_SCHEMA_PROPERTIES_TABLE = var.schema_properties_table_name
        GRAPL_USER_AUTH_TABLE         = var.user_auth_table
        GRAPL_TEST_USER_NAME          = var.test_user_name
        GRAPL_LOG_LEVEL               = var.rust_log # TODO: revisit
      }
    }

    service {
      name = "provisioner"
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
  }
}