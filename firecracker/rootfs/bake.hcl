target "rootfs-build" {
  context    = "."
  dockerfile = "Dockerfile"
  tags = [
    "rootfs-build:dev"
  ]
}