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