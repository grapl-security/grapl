# This job is to spin up infrastructure needed to run Grapl locally (e.g. Redis) that we don't necessarily want to deploy in production (because AWS will manage it)
job "grapl-local-infra" {
    datacenters = ["dc1"]

    type = "service"

    group "redis" {
        network {
            port "redis" {
                to = 6379
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

    group "dynamodb" {
        network {
            port "dynamodb" {
                to = 8000
            }
        }

        task "dynamodb" {
            driver = "docker"

            config {
                image = "amazon/dynamodb-local:latest"
                ports = ["dynamodb"]
            }
        }
    }

    group "sqs" {
        network {
            port "sqs" {
                to = 9324
            }
        }

        task "sqs" {
            driver = "docker"

            config {
                image = "graze/sqs-local:latest"
                ports = ["sqs"]
            }
        }
    }
}