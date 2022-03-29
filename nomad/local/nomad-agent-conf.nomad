
# This is not specified by default for nomad agent -dev
plugin_dir = "/opt/nomad/plugins"

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

    # We need net_admin for dnsmasq to work. Everything else should be default.
    # The list of default permissions can be found in https://www.nomadproject.io/docs/drivers/docker#allow_caps
    allow_caps = [
      # non-default options start here
      "net_admin",
      # default options start here
      "audit_write", "chown", "dac_override", "fowner", "fsetid", "kill", "mknod",
      "net_bind_service", "setfcap", "setgid", "setpcap", "setuid", "sys_chroot"
    ]
  }
}

plugin "firecracker-task-driver" {}

####################
# Client config
####################

client {
  meta = {
    # See constraint{} in plugin.nomad
    "is_grapl_plugin_host" = true

    # Turn on consul connect proxy debug logs. Consul connect sidecars now have access logs, etc.
    connect.log_level = "debug"
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
