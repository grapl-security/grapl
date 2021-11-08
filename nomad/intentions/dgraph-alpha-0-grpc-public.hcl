Kind = "service-intentions"
Name = "dgraph-alpha-0-grpc-public"
Sources = [
  {
    Name   = "analyzer-executor"
    Action = "allow"
  },
  {
    Name   = "engagement-creator"
    Action = "allow"
  },
  {
    Name   = "graph-merger"
    Action = "allow"
  },
  {
    Name   = "graphql-endpoint"
    Action = "allow"
  },
  # NOTE: a default catch-all based on the default ACL policy will apply to
  # unmatched connections and requests. Typically this will be DENY.
]