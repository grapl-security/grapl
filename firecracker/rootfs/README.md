# Build Firecracker RootFS

Here we use Packer to build a Firecracker RootFS. The general flow is as
follows:

- Spin up an EC2 instance, just like a normal build-AMI Packer job would do.
- Create an image file (`create_rootfs_image.sh`). It will be provisioned with
  Debian Bullseye.
- [NOT DONE YET] Copy the Grapl Plugin Bootstrap Client into the image and hook
  it up to systemd.
- The final `provisioner "file"` stanza of build-rootfs.pkr.hcl will download
  that .gz from the EC2 instance and dump it in `${GRAPL_ROOT}/dist/`.

Notably, while we use the amazon-ebs builder, we don't actually create an AMI
and we certainly don't _consume_ an AMI. Instead we simply use Packer as a handy
mechanism to interface with an EC2 build machine.
