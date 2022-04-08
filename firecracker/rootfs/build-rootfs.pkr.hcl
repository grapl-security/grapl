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

variable "ami_id" {
  description = <<EOF
Ubuntu AMI ID to build with. Must match var.region. We grab this one with:

AWS_REGION=us-east-1 aws ssm get-parameters --names \
  /aws/service/canonical/ubuntu/server-minimal/20.04/stable/current/amd64/hvm/ebs-gp2/ami-id \
  | jq .Parameters[0].Value
}
EOF
  type        = string
  default     = "ami-0623cccc9df636385"
}

variable "aws_profile" {
  description = "The AWS connection profile to use when creating the image"
  type        = string
  default     = env("AWS_PROFILE")
}

variable "dist_dir" {
  description = "This is the directory on the local workstation that the final image file will be deposited into"
  type        = string
}

variable "plugin_bootstrap_init_artifacts_dir" {
  description = <<EOF
A directory on the local workstation containing the built plugin-bootstrap-init
artifact (and its accompanying two .service files).
Basically: This is the dir created by `make dist/plugin-bootstrap-init`.
EOF
  type        = string
}

variable "image_name" {
  description = "The name of the artifact that will end up in var.dist_dir, excluding .tar.gz"
  type        = string
}

variable "debian_version" {
  description = "Which version of Debian to use with debootstrap."
  type        = string
  default     = "bullseye" # aka 11.0
}

########################################################################

locals {
  image_archive_filename    = "${var.image_name}.tar.gz"
  destination_in_dist       = "${var.dist_dir}/${local.image_archive_filename}"
  init_artifacts_dir_remote = "/home/ubuntu/${basename(var.plugin_bootstrap_init_artifacts_dir)}"

  # The base Debootstrap install takes up 252MB
  # The plugin-bootstrap-init built with RUST_BUILD=debug (e.g. locally) is
  # 223MB
  # Give it some buffer
  image_size_mb = 600
}

data "amazon-ami" "base-ami" {
  filters = {
    image-id = var.ami_id
  }
  owners = ["099720109477"] # Canonical / Ubuntu
  region = var.region
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
  region          = var.region
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

  # Prevents non-deterministic apt failures in install_dependencies.sh
  # https://github.com/grapl-security/issue-tracker/issues/880
  provisioner "shell" {
    inline = ["cloud-init status --wait"]
  }

  provisioner "file" {
    direction   = "upload"
    source      = "${path.root}/scripts"
    destination = "/home/ubuntu"
  }

  provisioner "shell" {
    inline = ["~/scripts/install_dependencies.sh"]
  }

  # Upload the bootstrap files so they can be embedded in the rootfs
  provisioner "file" {
    direction   = "upload"
    source      = "${var.plugin_bootstrap_init_artifacts_dir}"
    destination = "/home/ubuntu/"
  }

  provisioner "shell" {
    inline = [
      "~/scripts/create_rootfs_image.sh",
    ]
    environment_vars = [
      "IMAGE_NAME=${var.image_name}",
      "IMAGE_ARCHIVE_NAME=${local.image_archive_filename}",
      "DEBIAN_VERSION=${var.debian_version}",
      "PLUGIN_BOOTSTRAP_INIT_ARTIFACTS_DIR=${local.init_artifacts_dir_remote}",
      "SIZE_MB=${local.image_size_mb}",
    ]
  }

  # Grab the output from EC2 and copy it over into the Packer Host OS's `dist/`
  provisioner "file" {
    direction   = "download"
    source      = "/home/ubuntu/output/${local.image_archive_filename}"
    destination = local.destination_in_dist
  }


  # A lot of the errors you may encounter during this build are due to the
  # built image being too small (local.image_size_mb); this cleanup provisioner
  # may help users debug that.
  error-cleanup-provisioner "shell" {
    inline = ["df --human"]
  }
}
