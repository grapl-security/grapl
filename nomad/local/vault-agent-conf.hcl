log_level          = "Info"
log_requests_level = "info"
ui                 = true
api_addr           = "http://127.0.0.1:8200"

# We're not setting service_registration in dev, since it creates a circular dependency where vault relies on consul
# being up, and consul relies on vault being up. The right way to handle this is to bring up consul, bring up vault,
# update the consul config and then reload consul. Ideally, we'd have an api/cli command to do the registration for vault
# instead. Long-term we may re-visit this IF service registration of vault would be useful.

# For dev we use in-memory. Do NOT use in prod
storage "inmem" {}

telemetry {
  # metrics can be scraped from "/v1/sys/metrics"
  # Note that this will require a vault bearer token
  prometheus_retention_time = "30s"
}