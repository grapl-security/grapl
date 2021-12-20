# This setup is inspired by the following forum discussion:
# https://discuss.hashicorp.com/t/best-practices-for-testing-against-services-in-nomad-consul-connect/29022
# We'll submit integration tests to Nomad as Nomad jobs.

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

variable "deployment_name" {
  type        = string
  description = "The deployment name"
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

variable "_redis_endpoint" {
  type        = string
  description = "On which port can services find redis?"
}

variable "_kafka_endpoint" {
  type        = string
  description = "On which port can services find Kafka?"
}

variable "schema_properties_table_name" {
  type        = string
  description = "What is the name of the schema properties table?"
}

variable "test_user_name" {
  type        = string
  description = "The name of the test user"
}

variable "docker_user" {
  type        = string
  description = "The UID:GID pair to run as inside the Docker container"
}

variable "grapl_root" {
  type        = string
  description = "Where to find the Grapl repo on the host OS (where Nomad runs)."
}

locals {
  log_level = "DEBUG"

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


  # Prefer these over their `var` equivalents
  redis_endpoint = replace(var._redis_endpoint, "LOCAL_GRAPL_REPLACE_IP", attr.unique.network.ip-address)
  kafka_endpoint = replace(var._kafka_endpoint, "LOCAL_GRAPL_REPLACE_IP", attr.unique.network.ip-address)

  _redis_trimmed = trimprefix(local.redis_endpoint, "redis://")
  _redis         = split(":", local._redis_trimmed)
  redis_host     = local._redis[0]
  redis_port     = local._redis[1]
}

job "integration-tests" {
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
    }

    # Enable service discovery
    service {
      name = "rust-integration-tests"
      connect {
        sidecar_service {
          proxy {
            upstreams {
              # This is a hack, because IDK how to share locals across files
              destination_name = "dgraph-alpha-0-grpc-public"
              local_bind_port  = 9080
            }

            upstreams {
              destination_name = "model-plugin-deployer"
              local_bind_port  = 1000 # doesn't really matter
            }

            upstreams {
              destination_name = "plugin-registry"
              local_bind_port  = 1001 # doesn't really matter
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

      # This writes an env file that gets read by the task automatically
      template {
        data        = local.conditionally_defined_env_vars
        destination = "rust-integration-tests.env"
        env         = true
      }

      env {
        AWS_REGION      = var.aws_region
        DEPLOYMENT_NAME = var.deployment_name
        GRAPL_LOG_LEVEL = local.log_level
        # This is a hack, because IDK how to share locals across files
        #MG_ALPHAS                   = local.alpha_grpc_connect_str # TODO: Figure out how to do this
        MG_ALPHAS      = "localhost:9080"
        RUST_BACKTRACE = 1
        RUST_LOG       = local.log_level
        REDIS_ENDPOINT = local.redis_endpoint
        KAFKA_ENDPOINT = local.kafka_endpoint

        GRAPL_MODEL_PLUGIN_DEPLOYER_HOST = "0.0.0.0"
        GRAPL_MODEL_PLUGIN_DEPLOYER_PORT = "${NOMAD_UPSTREAM_PORT_model-plugin-deployer}"

        GRAPL_PLUGIN_REGISTRY_ADDRESS = "http://0.0.0.0:${NOMAD_UPSTREAM_PORT_plugin-registry}"
      }

      # Because Cargo does some... compiling... for some reason.... maybe.....
      resources {
        memory = 6000
      }
    }
  }

  group "python-integration-tests" {
    restart {
      # Make this a one-shot job. Absolute worst case, Buildkite reruns it for us.
      attempts = 0
    }

    network {
      mode = "bridge"
    }

    # Enable service discovery
    service {
      name = "python-integration-tests"
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

    task "python-integration-tests" {
      driver = "docker"
      user   = var.docker_user

      config {
        image = var.container_images["python-integration-tests"]
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
        data        = local.conditionally_defined_env_vars
        destination = "python-integration-tests.env"
        env         = true
      }

      env {
        AWS_REGION = "${var.aws_region}"

        # These environment vars need to exist but the values aren't actually exercised
        GRAPL_ANALYZER_MATCHED_SUBGRAPHS_BUCKET = "NOT_ACTUALLY_EXERCISED_IN_TESTS"
        GRAPL_ANALYZERS_BUCKET                  = "NOT_ACTUALLY_EXERCISED_IN_TESTS"
        GRAPL_MODEL_PLUGINS_BUCKET              = "NOT_ACTUALLY_EXERCISED_IN_TESTS"

        GRAPL_API_HOST                = "${NOMAD_UPSTREAM_IP_web-ui}"
        GRAPL_HTTP_FRONTEND_PORT      = "${NOMAD_UPSTREAM_PORT_web-ui}"
        GRAPL_TEST_USER_NAME          = "${var.test_user_name}"
        GRAPL_SCHEMA_PROPERTIES_TABLE = "${var.schema_properties_table_name}"

        HITCACHE_ADDR     = "${local.redis_host}"
        HITCACHE_PORT     = "${local.redis_port}"
        MESSAGECACHE_ADDR = "${local.redis_host}"
        MESSAGECACHE_PORT = "${local.redis_port}"
        IS_RETRY          = "False"

        DEPLOYMENT_NAME = "${var.deployment_name}"
        GRAPL_LOG_LEVEL = "${local.log_level}"
        MG_ALPHAS       = "localhost:9080"

      }

      resources {
        memory = 1024
      }
    }
  }

}

