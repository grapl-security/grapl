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
  # compat disabled because hcl formatting breaks on the '.'. However this is what is used in AWS
  # disable_compat_1.9 = true
  prometheus_retention_time = "30s"
}

ports {
  dns = 53
}

# Set dns recursors so that when consul dns is queried for anything besides .consul, we can still get valid dns entries.
# We're using Google DNS as the primary, with a fallback to cloudflare dns. These can be switched out as necessary
recursors = ["8.8.8.8", "8.8.4.4", "1.1.1.1"]
