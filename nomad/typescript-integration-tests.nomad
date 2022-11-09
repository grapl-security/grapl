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

variable "test_user_name" {
  type        = string
  description = "The name of the test user"
}

variable "test_user_password_secret_id" {
  type        = string
  description = "The SecretsManager SecretID for the test user's password"
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

    service {
      name = "typescript-integration-tests"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "web-ui"
              local_bind_port  = 1006
            }
          }
        }
      }
    }

    task "typescript-integration-tests" {
      driver = "docker"

      config {
        image = var.container_images["typescript-integration-tests"]
      }

      # This writes an env file that gets read by the task automatically
      template {
        data        = var.aws_env_vars_for_local
        destination = "aws-env-vars-for-local.env"
        env         = true
      }

      env {
        AWS_REGION                         = var.aws_region
        GRAPL_TEST_USER_NAME               = var.test_user_name
        GRAPL_TEST_USER_PASSWORD_SECRET_ID = var.test_user_password_secret_id
        GRAPL_WEB_UI_ENDPOINT_ADDRESS      = "http://${NOMAD_UPSTREAM_ADDR_web-ui}"
      }
      resources {
        memory = 5000
      }
    }
  }
}