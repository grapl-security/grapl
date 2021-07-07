packer {
  required_version = ">= 0.12.0"
}

variable "build_ami" {
  description = "Whether or not to actually build an AMI. Set to `false` for doing testing"
  type        = bool
  default     = true
}

variable "ami_name_prefix" {
  type    = string
  default = "nomad-consul"
}

variable "aws_region" {
  type    = string
  default = "us-east-1"
}

variable "consul_module_version" {
  type    = string
  default = "v0.10.1"
}

variable "consul_version" {
  type    = string
  default = "1.9.6"
}

variable "nomad_version" {
  type    = string
  default = "1.1.1"
}

variable "terraform_aws_nomad_version" {
  type = string
  default = "v0.9.1"
}

locals {
  formatted_timestamp = formatdate("YYYYMMDDhhmmss", timestamp())
  copy_ami_to_regions = [
    "us-east-2",
    "us-west-1",
    "us-west-2",
  ]
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
  region      = "${var.aws_region}"
}

source "amazon-ebs" "amazon-linux-2-amd64-ami" {
  ami_description = "An Amazon Linux 2 x86_64 AMI that has Nomad and Consul installed."
  ami_name        = "${var.ami_name_prefix}-amazon-linux-2-amd64-${local.formatted_timestamp}"
  instance_type   = "t2.micro"
  region          = "${var.aws_region}"
  # Where we copy it after it's built
  ami_regions     = local.copy_ami_to_regions 
  source_ami      = data.amazon-ami.amazon-linux-2-x86_64.id
  ssh_username    = "ec2-user"
  skip_create_ami = "${var.build_ami == true ? false : true}"
}

build {
  sources = ["source.amazon-ebs.amazon-linux-2-amd64-ami"]

  provisioner "shell" {
    inline = ["sudo yum install -y git"]
    only   = ["amazon-linux-2-amd64-ami"]
  }

  provisioner "shell" {
    inline       = ["mkdir -p /tmp/terraform-aws-nomad"]
    pause_before = "30s"
  }

  # As recommended in https://github.com/hashicorp/terraform-aws-nomad/tree/master/examples/nomad-consul-ami readme
  provisioner "shell" {
    inline = [
      "git clone --branch ${var.terraform_aws_nomad_version} https://github.com/hashicorp/terraform-aws-nomad.git /tmp/terraform-aws-nomad",
      "/tmp/terraform-aws-nomad/modules/install-nomad/install-nomad --version ${var.nomad_version}"
    ]
  }

  provisioner "shell" {
    environment_vars = [
      "NOMAD_VERSION=${var.nomad_version}", 
      "CONSUL_VERSION=${var.consul_version}", 
      "CONSUL_MODULE_VERSION=${var.consul_module_version}"
    ]
    script           = "${path.root}/setup_nomad_consul.sh"
  }

  post-processor "manifest" {
    output = "nomad-server.packer-manifest.json"
  }
}
