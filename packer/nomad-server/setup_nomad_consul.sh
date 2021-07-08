#!/bin/sh
set -euo pipefail

# As recommended in https://github.com/hashicorp/terraform-aws-nomad/tree/master/examples/nomad-consul-ami readme
git clone --branch ${TERRAFORM_AWS_NOMAD_TAG} https://github.com/hashicorp/terraform-aws-nomad.git /tmp/terraform-aws-nomad
/tmp/terraform-aws-nomad/modules/install-nomad/install-nomad --version "${NOMAD_VERSION}"

git clone --branch "${TERRAFORM_AWS_CONSUL_TAG}"  https://github.com/hashicorp/terraform-aws-consul.git /tmp/terraform-aws-consul
/tmp/terraform-aws-consul/modules/install-consul/install-consul --version "${CONSUL_VERSION}"
