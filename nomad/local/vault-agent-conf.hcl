log_level = "Info"
log_requests_level = "info"
ui = true

api_addr = "http://127.0.0.1:8200"

service_registration "consul" {
  address = "127.0.0.1:8500"
}

# For dev we use in-memory. Do NOT use in prod
storage "inmem" {}

telemetry {
  # metrics can be scraped from "/v1/sys/metrics"
  # Note that this will require a vault bearer token
  prometheus_retention_time = "30s"
}