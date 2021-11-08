Kind = "service-intentions"
Name = "dgraph-alpha-0-grpc-private"
Sources = [
  {
    Name   = "dgraph-zero-0-grpc-private"
    Action = "allow"
  },

  # NOTE: a default catch-all based on the default ACL policy will apply to
  # unmatched connections and requests. Typically this will be DENY.
]