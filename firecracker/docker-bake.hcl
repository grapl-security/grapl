target "microvm-kernel" {
  context = "."
  dockerfile = "Dockerfile"
  target = "firecracker-vm"
  output =[
    "type=local,dest=."
  ]
}