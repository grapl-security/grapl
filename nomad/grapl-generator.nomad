variable "tenant_id" {
    type = string
    description = "The opaque tenant id."
}

variable "generator_ingest" {
    type = string
    description = "The expected data ingest type. e.g. sysmon."
}

variable "generator_artifact_url" {
    type = string
    description = "The url of the generator artifact to run. Supported formats: http(s), git, hg, and s3."
}

variable "generator_count" {
    type = number
    default = 1
}

job "grapl-tenant-generator" {
    namespace = "generator-${var.tenant_id}-${var.generator_ingest}"
    datacenters = ["dc1"]

    type = "service"

    update {
        # Automatically promotes to canaries if all canaries are healthy during an update / deployment 
        auto_promote = true
        # Auto reverts to the last stable job variant if the update fails 
        auto_revert = true
        # Spins up a "canary" instance of potentially destructive updates, validates that they are healthy, then promotes the instance to update
        canary = 1
        max_parallel = 1
        # The min amount of reported "healthy" time before a instance is considered healthy and an allocation is opened up for further updates
        min_healthy_time = "15s"
    }

    group "generator" {
        network {
            port "generator_receiver" { }
        }

        count = var.generator_count

        task "tenant-generator" {
            driver = "exec"

            artifact {
                source = var.generator_artifact_url
                destination = "local/generator"
                mode = "file"
            }

            env {
                TENANT_ID = "${var.tenant_id}"
                INGEST_TYPE = "${var.generator_ingest}"
            }

            config {
                command = "generator"
            }

            service {
                name = "generator-${var.tenant_id}-${var.generator_ingest}"
                port = "generator_receiver"
                tags = ["generator-${var.tenant_id}"]
                canary_tags = ["canary-generator-${var.tenant_id}"]
            }
        }
    }
}