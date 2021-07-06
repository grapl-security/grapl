
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

locals { 
  # We append a timestamp to the AMI name to create unique names.
  formatted_timestamp = formatdate("YYYYMMDDhhmmss", timestamp())

  # Copied from src/python/graplctl/graplctl/swarm/lib.py
  region_to_base_ami_id = {
    us-east-1 = "ami-0947d2ba12ee1ff75"
  }

  build_region = "us-east-1"
  copy_ami_to_regions = [
    "us-east-2",
    "us-west-1",
    "us-west-2",
  ]
}

source "amazon-ebs" "nomad-server-image" {
  ami_name      = "grapl-nomad-server-${local.formatted_timestamp}"
  instance_type = "t2.micro"
  # Where it's built and made available
  region        = local.build_region
  # Where we copy it after it's built
  ami_regions   = local.copy_ami_to_regions 
  source_ami_filter {
    filters = {
      image-id            = local.region_to_base_ami_id[local.build_region]
      root-device-type    = "ebs"
      virtualization-type = "hvm"
    }
    most_recent = true
    owners = ["amazon"]
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
    volume_size           = 15
    delete_on_termination = false
  }
}

build {
  sources = [
    "source.amazon-ebs.nomad-server-image"
  ]

  post-processor "manifest" {
    output = "packer-manifest.json" # The default value; just being explicit
  }

  provisioner "file" {
    source = "${path.root}/files"
    destination = "/tmp/"
  }

  provisioner "shell" {
    execute_command = "{{.Vars}} sudo --preserve-env bash -x '{{.Path}}'"
    scripts = [
      "${path.root}/scripts/packages.sh",
      "${path.root}/scripts/download-files.sh",
      "${path.root}/scripts/consul.sh",
      "${path.root}/scripts/nomad.sh",
    ]
  }
}
