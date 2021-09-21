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

plugin "raw_exec" {
  config {
    enabled = true
  }
}