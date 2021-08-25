variable "container_registry" {
  type        = string
  default     = "localhost:5000"
  description = "The container registry in which we can find Grapl services."
}

# The following variables are all-caps to clue in users that they're
# imported from `local-grapl.env`.
variable "FAKE_AWS_ACCESS_KEY_ID" {
  type        = string
  description = "Fake AWS Access Key ID for Localstack and clients"
}

variable "FAKE_AWS_SECRET_ACCESS_KEY" {
  type        = string
  description = "Fake AWS Secret Access Key for Localstack and clients"
}

variable "LOCALSTACK_PORT" {
  type        = string
  description = "Port for Localstack"
}

variable "LOCALSTACK_HOST" {
  type        = string
  description = "External hostname for Localstack"
}


####################
# Jobspecs
####################

# This job is to spin up infrastructure needed to run Grapl locally (e.g. Redis) that we don't necessarily want to deploy in production (because AWS will manage it)
job "grapl-local-infra" {
  datacenters = ["dc1"]

  type = "service"

  group "redis" {
    network {
      port "redis" {
        static = 6379
      }
    }

    task "redis" {
      driver = "docker"

      config {
        image = "redis:latest"
        ports = ["redis"]
      }
    }
  }

  group "localstack" {
    network {
      mode = "bridge"

      port "localstack" {
        static = var.LOCALSTACK_PORT
      }
    }

    task "localstack" {
      # TODO: How to do health check?

      driver = "docker"

      config {
        # Once we move to Kafka, we can go back to the non-fork.
        image = "${var.container_registry}/grapl/localstack:latest"
        # Was running into this: https://github.com/localstack/localstack/issues/1349
        memory_hard_limit = 2048
        ports             = ["localstack"]
        privileged        = true
        volumes = [
          "/var/run/docker.sock:/var/run/docker.sock"
        ]
        # Do not set Docker's `network_mode` if you specify a group network_mode
        network_aliases = [
          var.LOCALSTACK_HOST
        ]
      }

      env {
        EDGE_PORT         = var.LOCALSTACK_PORT
        HOSTNAME_EXTERNAL = var.LOCALSTACK_HOST
        SERVICES          = "apigateway,cloudwatch,dynamodb,ec2,events,iam,lambda,logs,s3,secretsmanager,sns,sqs"
        DEBUG             = 1
        LAMBDA_EXECUTOR   = docker-reuse
        # TODO: MAIN_CONTAINER_NAME = 
        LAMBDA_DOCKER_NETWORK = grapl-network
        # TODO? DATA_DIR =
        SQS_PROVIDER = elasticmq
      }
    }
  }

  group "ratel" {
    network {
      port "ratel" {
        static = 8000
      }
    }

    task "ratel" {
      driver = "docker"

      config {
        image = "dgraph/ratel:latest"
        ports = ["ratel"]
      }
    }
  }
}