variable "TAG" {
  default = "latest"
}

variable "PROFILE" {
  # Cargo profile, either 'debug' or 'release'
  default = "debug"
}

target "_services_common" {
  context    = "dist/${PROFILE}"
  dockerfile = "../../dist.Dockerfile"
}

group "default" {
  targets = [
    "sysmon-generator",
    "osquery-generator",
    "node-identifier",
    "node-identifier-retry",
    "graph-merger",
    "analyzer-dispatcher",
    "grapl-web-ui"
  ]
}

target "sysmon-generator" {
  inherits = ["_services_common"]
  target   = "sysmon-generator"
  tags     = ["grapl/sysmon-generator:${TAG}"]
}

target "osquery-generator" {
  inherits = ["_services_common"]
  target   = "osquery-generator"
  tags     = ["grapl/osquery-generator:${TAG}"]
}

target "node-identifier" {
  inherits = ["_services_common"]
  target   = "node-identifier"
  tags     = ["grapl/node-identifier"]
}

target "node-identifier-retry" {
  inherits = ["_services_common"]
  target   = "node-identifier-retry"
  tags     = ["grapl/node-identifier-retry:${TAG}"]
}

target "graph-merger" {
  inherits = ["_services_common"]
  target   = "graph-merger"
  tags     = ["grapl/graph-merger:${TAG}"]
}

target "analyzer-dispatcher" {
  inherits = ["_services_common"]
  target   = "analyzer-dispatcher"
  tags     = ["grapl/analyzer-dispatcher:${TAG}"]
}

target "grapl-web-ui" {
  inherits = ["_services_common"]
  target   = "grapl-web-ui"
  tags     = ["grapl/grapl-web-ui:${TAG}"]
}

target "int-tests" {
  context    = "dist"
  dockerfile = "../integration-tests.Dockerfile"
  tags       = ["grapl/rust-integration-tests:${TAG}"]
}
