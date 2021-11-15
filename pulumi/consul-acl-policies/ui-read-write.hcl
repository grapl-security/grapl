# ACL permissions for read-only access to the UI
service_prefix "" {
  policy = "read"
}
key_prefix "" {
  policy = "write"
}
node_prefix "" {
  policy = "read"
}

acl = "write"