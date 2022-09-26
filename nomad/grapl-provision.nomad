# This job is separate from `grapl-core.nomad` because
# we want to have this job depend on the successful, complete startup of 
# grapl-core.
# There are more-robust ways to do this, that could bring
# `grapl-provisioner` back  into the `grapl-core` fold, but
# this will get the job done for the time being.

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

variable "container_images" {
  type        = map(string)
  description = <<EOF
  A map of $NAME_OF_TASK to the URL for that task's docker image ID.
  (See DockerImageId in Pulumi for further documentation.)
EOF
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

variable "test_user_password_secret_id" {
  type        = string
  description = "The SecretsManager SecretID for the test user's password"
}

variable "py_log_level" {
  type        = string
  description = "Controls the logging behavior of Python-based services."
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
        image      = var.container_images["provisioner"]
        entrypoint = ["/bin/bash", "-c", "-o", "errexit", "-o", "nounset", "-c"]
        command    = "./provisioner.pex"
      }

      lifecycle {
        hook = "poststart"
        # Ephemeral, not long-lived
        sidecar = false
      }

      # This writes an env files that gets read by nomad automatically
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
        AWS_DEFAULT_REGION                 = var.aws_region
        DGRAPH_DROP_ALL_DATA               = 1
        GRAPL_SCHEMA_TABLE                 = var.schema_table_name
        GRAPL_SCHEMA_PROPERTIES_TABLE      = var.schema_properties_table_name
        GRAPL_USER_AUTH_TABLE              = var.user_auth_table
        GRAPL_TEST_USER_NAME               = var.test_user_name
        GRAPL_TEST_USER_PASSWORD_SECRET_ID = var.test_user_password_secret_id
        GRAPL_LOG_LEVEL                    = var.py_log_level
      }
    }

    service {
      name = "provisioner"
      connect {
        sidecar_service {
          proxy {}
        }
      }
    }
  }
}