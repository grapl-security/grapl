log_level = "Info"
log_requests_level = "info"
ui = true

listener "tcp" {
  address     = "127.0.0.1:8200"
  tls_disable = 1
}

service_registration "consul" {
  address = "127.0.0.1:8500"
}

telemetry {
  # metrics can be scraped from "/v1/sys/metrics"
  # Note that this will require a vault bearer token
  prometheus_retention_time = "30s"
}