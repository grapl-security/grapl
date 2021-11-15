# ACL permissions for read-only access to the UI
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