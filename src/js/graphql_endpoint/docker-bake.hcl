variable "TAG" {
  default = "latest"
}

group "default" {
  targets = [
    "graphql_endpoint",
  ]
}

target "graphql_endpoint" {
  context    = "."
  dockerfile = "dist.Dockerfile"
  tags       = ["grapl/graphql-endpoint:${TAG}"]
}

