Kind = "service-intentions"
Name = "graphql-endpoint"
Sources = [
  {
    Name   = "web-ui"
    Action = "allow"
  },

  # NOTE: a default catch-all based on the default ACL policy will apply to
  # unmatched connections and requests. Typically this will be DENY.
]