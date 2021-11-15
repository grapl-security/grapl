# Policy used for consul agents, in particular nomad agents.

# Allow the agent to register its own node in the Catalog and update its network coordinates, update node health checks
# and have write access to its own agent config files.
node "" {
  policy = "write"
}

# Allows the agent to detect and diff services registered to itself. This is used during
# anti-entropy to reconcile difference between the agents knowledge of registered
# services and checks in comparison with what is known in the Catalog.
service_prefix "" {
  policy = "write"
}

# only necessary if using dns prepared queries
query_prefix "" {
  policy = "read"
}
