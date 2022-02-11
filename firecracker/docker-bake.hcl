target "microvm-kernel" {
  context = "."
  dockerfile = "Dockerfile"
  target = "build-firecracker-vm"
  output =[
    "type=local,dest=."
  ]
}