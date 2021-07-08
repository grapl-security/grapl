variable "build_ami" {
  description = "Whether or not to actually build an AMI. Set to `false` for doing testing"
  type        = bool
  default     = true
}

variable "is_server" {
  description = "Is this image for the Nomad/Consul server, or its client?"
  type        = bool
}

variable "terraform_consul_module_tag" {
  description = "Version tag of terraform-aws-consul to checkout"
  type        = string
  default     = "v0.10.1"
}

variable "terraform_aws_nomad_tag" {
  description = "Version tag of terraform-aws-nomad to checkout"
  type        = string
  default     = "v0.9.1"
}

variable "consul_version" {
  description = "Version of consul to use"
  type        = string
  default     = "1.9.6"
}

variable "nomad_version" {
  description = "Version of Nomad to use"
  type        = string
  default     = "1.1.1"
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

  formatted_timestamp = formatdate("YYYYMMDDhhmmss", timestamp())
  ami_build_region = "us-east-1"
  copy_ami_to_regions = [
    "us-east-2",
    "us-west-1",
    "us-west-2",
  ]
  image_name = var.is_server ? "grapl-nomad-consul-server" : "grapl-nomad-consul-client"
}

data "amazon-ami" "amazon-linux-2-x86_64" {
  filters = {
    architecture                       = "x86_64"
    "block-device-mapping.volume-type" = "gp2"
    name                               = "*amzn2-ami-hvm-*"
    root-device-type                   = "ebs"
    virtualization-type                = "hvm"
  }
  most_recent = true
  owners      = ["amazon"]
  region      = local.ami_build_region
}

source "amazon-ebs" "amazon-linux-2-amd64-ami" {
  ami_description = "An Amazon Linux 2 x86_64 AMI that has Nomad and Consul installed."
  ami_name        = "${var.image_name}-amazon-linux-2-amd64-${local.formatted_timestamp}"
  instance_type   = "t2.micro"
  region          = local.ami_build_region
  # Where we copy it after it's built
  ami_regions  = local.copy_ami_to_regions
  source_ami   = data.amazon-ami.amazon-linux-2-x86_64.id
  ssh_username = "ec2-user"

  tag {
    key   = "BuiltBy"
    value = "Packer ${packer.version}"
  }

  # Filter out any variable-based tags that are unset.
  dynamic "tag" {
    for_each = { for key, value in local.tags_from_vars : key => value if value != "" }
    content {
      key   = tag.key
      value = tag.value
    }
  }

  skip_create_ami = "${var.build_ami == true ? false : true}"
}

build {
  sources = ["source.amazon-ebs.amazon-linux-2-amd64-ami"]

  provisioner "shell" {
    inline = [
      "sudo yum install -y git",
    ]
  }

  provisioner "shell" {
    environment_vars = [
      "NOMAD_VERSION=${var.nomad_version}",
      "CONSUL_VERSION=${var.consul_version}",
      "TERRAFORM_AWS_CONSUL_TAG=${var.terraform_consul_module_tag}",
      "TERRAFORM_AWS_NOMAD_TAG=${var.terraform_nomad_module_tag}",
    ]
    script = "${path.root}/setup_nomad_consul.sh"
  }

  # Copy Nomad configs to a temp spot on its way to `/opt/nomad/config`
  provisioner "file" {
    source      = "${path.root}/nomad-config"
    destination = "/tmp/"
  }

  # Copy Consul configs to a temp spot on its way to `/opt/consul/config`
  provisioner "file" {
    source      = "${path.root}/consul-config"
    destination = "/tmp/"
  }

  provisioner "shell" {
    inline = [
      "sudo cp /tmp/nomad-config/* /opt/nomad/config",
      "sudo cp /tmp/consul-config/* /opt/consul/config",
    ]
  }

  post-processor "manifest" {
    output = "${var.image_name}.packer-manifest.json"
  }
}
