packer {
  required_plugins {
    amazon = {
      version = ">= 1.0.6"
      source  = "github.com/hashicorp/amazon"
    }
  }
}

variable "instance_type" {
  description = "The type of instance used to *build* the image"
  type        = string
  default     = "m5.xlarge"
}

variable "region" {
  description = "In which region to build the rootfs. Must match the Base AMI's region."
  type        = string
  default     = "us-east-1"
}

variable "aws_profile" {
  description = "The AWS connection profile to use when creating the image"
  type        = string
  default     = env("AWS_PROFILE")
}

variable "dist_folder" {
  description = "This is the directory on the local workstation that the final image file will be deposited into"
  type        = string
}

variable "image_name" {
  description = "The name of the artifact that will end up in var.dist_folder, excluding .tar.gz"
  type        = string
  default     = "firecracker_rootfs"
}

variable "debian_version" {
  description = "Which version of Debian to use with debootstrap."
  type        = string
  default     = "bullseye" # aka 11.0
}

########################################################################

locals {
  image_archive_filename = "${var.image_name}.tar.gz"
  destination_in_dist    = "${var.dist_folder}/${local.image_archive_filename}"
}

data "amazon-ami" "base-ami" {
  filters = {
    # official amd64 Ubuntu Focal (20.04 LTS) image on us-east-1
    # per https://cloud-images.ubuntu.com/locator/ec2/
    image-id = "ami-01896de1f162f0ab7"
  }
  owners = ["099720109477"] # Canonical / Ubuntu
  region = "${var.region}"
}

source "amazon-ebs" "grapl-build-rootfs" {
  # We don't actually want an AMI; We just use this environment to spin up
  # a `debootstrap` run, and copy those artifacts back to the machine running
  # Packer
  skip_create_ami = true

  # These fields aren't really that useful, given that we're not storing
  # the AMI itself
  ami_description = "Grapl Build Environment for RootFS"
  ami_name        = "grapl-build-rootfs-linux-x86_64"
  instance_type   = "${var.instance_type}"
  region          = "${var.region}"
  source_ami      = "${data.amazon-ami.base-ami.id}"
  ssh_username    = "ubuntu"
  profile         = "${var.aws_profile}"

  metadata_options {
    http_endpoint               = "enabled"
    http_tokens                 = "required"
    http_put_response_hop_limit = 1
  }

}

build {
  sources = ["source.amazon-ebs.grapl-build-rootfs"]

  provisioner "file" {
    direction   = "upload"
    source      = "${path.root}/scripts"
    destination = "/home/ubuntu"
  }

  provisioner "shell" {
    inline = ["~/scripts/install_dependencies.sh"]
  }

  provisioner "shell" {
    inline = [
      "~/scripts/create_rootfs_image.sh",
    ]
    environment_vars = [
      "IMAGE_NAME=${var.image_name}",
      "IMAGE_ARCHIVE_NAME=${local.image_archive_filename}",
      "DEBIAN_VERSION=${var.debian_version}",
      "SIZE_MB=400",
    ]
  }

  # Grab the output from EC2 and copy it over into the Packer Host OS's `dist/`
  provisioner "file" {
    direction   = "download"
    source      = "/home/ubuntu/output/${local.image_archive_filename}"
    destination = local.destination_in_dist
  }
}
