# ACL permissions for read-only access to the UI

# "" is a wildcard match.
service_prefix "" {
  policy = "read"
}
key_prefix "" {
  policy = "read"
}
node_prefix "" {
  policy = "read"
}

acl = "read"