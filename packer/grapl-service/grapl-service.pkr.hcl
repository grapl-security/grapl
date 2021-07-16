
variable "volume_size" {
  description = "Size of EBS volume to mount"
  type        = number
  default     = 15
}

variable "build_ami" {
  description = "Whether or not to actually build an AMI. Set to `false` for doing testing"
  type        = bool
  default     = true
}

variable "git_sha" {
  description = "The git SHA of the commit the AMI is being generated from. If present, will be used to tag the AMI."
  type        = string
  default     = env("GIT_SHA")
}

variable "git_branch" {
  description = "The git branch the AMI is being generated from. If present, will be used to tag the AMI."
  type        = string
  default     = env("GIT_BRANCH")
}

variable "buildkite_build_number" {
  description = "The build number of the Buildkite pipeline this AMI was built in. If present, will be used to tag the AMI."
  type        = string
  default     = env("BUILDKITE_BUILD_NUMBER")
}

variable "aws_profile" {
  description = "The AWS connection profile to use when creating the image"
  type        = string
  default     = env("AWS_PROFILE")
}

locals {
  # These are various metadata tags we can add to the resulting
  # AMI. If any are unset (like the Buildkite build number, if
  # building outside of Buildkite), those tags will be filtered out
  # and not added to the AMI.
  tags_from_vars = {
    "git:sha"                = "${var.git_sha}"
    "git:branch"             = "${var.git_branch}"
    "buildkite:build_number" = "${var.buildkite_build_number}"
  }
}

packer {
  required_plugins {
    amazon = {
      version = ">= 0.0.1"
      source  = "github.com/hashicorp/amazon"
    }
  }
}

source "amazon-ebs" "grapl-base-service-image" {
  ami_name      = "grapl-base-service-aws"
  instance_type = "t2.micro"
  region        = "us-east-2"
  source_ami_filter {
    filters = {
      image-id            = "ami-0a8d76686b0740efe"
      root-device-type    = "ebs"
      virtualization-type = "hvm"
    }
    most_recent = true
    owners      = ["amazon"]
  }

  ssh_username = "ec2-user"

  tag {
    key   = "BuiltBy"
    value = "Packer ${packer.version}"
  }

  dynamic "tag" {
    for_each = { for key, value in local.tags_from_vars : key => value if value != "" }
    content {
      key   = tag.key
      value = tag.value
    }
  }

  skip_create_ami = "${var.build_ami == true ? false : true}"

  launch_block_device_mappings {
    device_name           = "/dev/xvda"
    volume_type           = "gp2"
    volume_size           = "${var.volume_size}"
    delete_on_termination = false
  }
}

build {
  sources = [
    "source.amazon-ebs.grapl-base-service-image"
  ]

  provisioner "file" {
    source      = "files"
    destination = "/tmp/"
  }

  provisioner "shell" {
    execute_command = "{{.Vars}} sudo --preserve-env bash -x '{{.Path}}'"
    scripts = [
      "scripts/packages.sh",
      "scripts/download-files.sh",
      "scripts/consul.sh",
      "scripts/nomad.sh",
    ]
  }
}
