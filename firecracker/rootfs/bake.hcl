target "rootfs-build" {
  context    = "."
  dockerfile = "Dockerfile.rootfs"
  tags = [
    "rootfs-build:dev"
  ]
}