# This setup is inspired by the following forum discussion:
# https://discuss.hashicorp.com/t/best-practices-for-testing-against-services-in-nomad-consul-connect/29022
# We'll submit integration tests to Nomad as Nomad jobs.
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

variable "aws_region" {
  type = string
}

variable "container_images" {
  type        = map(string)
  description = <<EOF
  A map of $NAME_OF_TASK to the URL for that task's docker image ID.
  (See DockerImageId in Pulumi for further documentation.)
EOF
}

variable "docker_user" {
  type        = string
  description = "The UID:GID pair to run as inside the Docker container"
}

variable "grapl_root" {
  type        = string
  description = "Where to find the Grapl repo on the host OS (where Nomad runs)."
}

variable "schema_properties_table_name" {
  type        = string
  description = "What is the name of the schema properties table?"
}

variable "test_user_name" {
  type        = string
  description = "The name of the test user"
}

variable "test_user_password_secret_id" {
  type        = string
  description = "The SecretsManager SecretID for the test user's password"
}


locals {
  log_level = "DEBUG"
}

job "typescript-integration-tests" {
  datacenters = ["dc1"]
  type        = "batch"
  parameterized {}

  reschedule {
    # Make this a one-shot job
    attempts = 0
  }

  # Specifies that this job is the most high priority job we have; nothing else should take precedence
  priority = 100

  group "typescript-integration-tests" {
    restart {
      # Make this a one-shot job. Absolute worst case, Buildkite reruns it for us.
      attempts = 0
    }

    network {
      mode = "bridge"
    }

    # Enable service discovery
    service {
      name = "typescript-integration-tests"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "web-ui"
              local_bind_port  = 1234
            }
            upstreams {
              destination_name = "plugin-registry"
              local_bind_port  = 1001
            }

          }
        }
      }
    }

    task "typescript-integration-tests" {
      driver = "docker"
      user   = var.docker_user

      config {
        image = var.container_images["typescript-integration-tests"]
        # Pants caches requirements per-user. So when we run a Docker container
        # with the host's userns, this lets us reuse the pants cache.
        # (This descreases runtime on my personal laptop from 390s to 190s)
        userns_mode = "host"

        mount {
          # Just to clarify, this is all Docker-verbiage mounts and binds.
          # Nothing Nomad-y about it.
          type     = "bind"
          source   = var.grapl_root
          target   = "/mnt/grapl-root"
          readonly = false
        }

        mount {
          type     = "volume"
          target   = "/mnt/pants-cache"
          source   = "pants-cache-volume"
          readonly = false
          volume_options {
            # Upon initial creation of this volume, *do* copy in the current
            # contents in the Docker image.
            no_copy = false
          }
        }
      }

      # This writes an env file that gets read by the task automatically
      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      env {
        AWS_REGION = "${var.aws_region}"

        GRAPL_API_HOST                     = "${NOMAD_UPSTREAM_IP_web-ui}"
        GRAPL_HTTP_FRONTEND_PORT           = "${NOMAD_UPSTREAM_PORT_web-ui}"
        GRAPL_TEST_USER_NAME               = var.test_user_name
        GRAPL_TEST_USER_PASSWORD_SECRET_ID = var.test_user_password_secret_id
        GRAPL_SCHEMA_PROPERTIES_TABLE      = var.schema_properties_table_name

        IS_RETRY = "False"

        GRAPL_LOG_LEVEL = local.log_level

      }

      resources {
        memory = 1024
      }
    }
  }

}
