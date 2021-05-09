from __future__ import annotations

from typing import Optional, Sequence

import pulumi_aws as aws
from pulumi_aws.ec2 import outputs
from typing_extensions import Protocol

import pulumi


class SwarmConnectable(Protocol):
    ingress: pulumi.Input[Sequence[outputs.SecurityGroupIngress]]
    egress: pulumi.Input[Sequence[outputs.SecurityGroupEgress]]
