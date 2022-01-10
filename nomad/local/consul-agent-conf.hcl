acl = {
  enabled = true
  # Set allow by default for initial bootstrapping. Switch to "deny" when we're ready
  default_policy = "allow"
  # tokens should be persisted to disk and reloaded when agent restarts
  enable_token_persistence = true
  # if agent can't read policies or tokens from leader, use any cached ACLs even if TTLs are expired. Everything else is
  # denied
  down_policy = "extend-cache"
}

telemetry {
  # Enable metrics for consul
  # metrics path is /v1/agent/metrics?format=prometheus
  disable_compat_1.9 = true
  prometheus_retention_time = "30s"
}
