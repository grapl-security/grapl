####################
# Plugin configs
####################

plugin "docker" {
  # https://www.nomadproject.io/docs/drivers/docker#plugin-options
  config {
    allow_privileged = true

    volumes {
      # Required for the bind mount for docker.sock
      enabled = true
    }
  }
}


####################
# Telemetry configs
####################
telemetry {
  # enable metrics for nomad
  # metrics path is /v1/metrics?format=prometheus
  collection_interval        = "1s"
  disable_hostname           = true
  prometheus_metrics         = true
  publish_allocation_metrics = true
  publish_node_metrics       = true
}
