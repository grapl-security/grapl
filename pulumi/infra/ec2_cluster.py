from typing import Dict, List, Optional, Sequence

import pulumi_aws as aws

import pulumi
from pulumi import Output


class Ec2Cluster(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        # For quorum purposes, you probably want something >=3 and odd.
        num_instances: int,
        ami: str,
        instance_type: str,
        iam_instance_profile: aws.iam.InstanceProfile,
        vpc_private_subnet: aws.ec2.Subnet,
        vpc_security_group_ids: Sequence[Output[str]],
        instance_tags: Dict[str, str],
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:Ec2ClusterResource", name=name, props=None, opts=opts)
        child_opts = pulumi.ResourceOptions(parent=self)

        self.instances = self._build_instances(
            name=f"{name}-cluster",
            ami=ami,
            instance_type=instance_type,
            num_instances=num_instances,
            iam_instance_profile=iam_instance_profile,
            vpc_private_subnet=vpc_private_subnet,
            vpc_security_group_ids=vpc_security_group_ids,
            instance_tags=instance_tags,
            child_opts=child_opts,
        )
        self.register_outputs(
            {
                "instances": self.instances,
            }
        )

    @staticmethod
    def _build_instances(
        name: str,
        ami: str,
        instance_type: str,
        num_instances: int,
        iam_instance_profile: aws.iam.InstanceProfile,
        vpc_private_subnet: aws.ec2.Subnet,
        vpc_security_group_ids: Sequence[Output[str]],
        instance_tags: Dict[str, str],
        child_opts: pulumi.ResourceOptions,
    ) -> List[aws.ec2.Instance]:
        instances = []
        for i in range(0, num_instances):
            instance_tags["Name"] = f"{name}-{i}"
            instance = aws.ec2.Instance(
                f"ec2-inst-{name}-{i}",
                ami=ami,
                instance_type=instance_type,
                subnet_id=vpc_private_subnet.id,
                vpc_security_group_ids=vpc_security_group_ids,
                credit_specification=aws.ec2.InstanceCreditSpecificationArgs(
                    cpu_credits="unlimited",
                ),
                tags=instance_tags,
                iam_instance_profile=iam_instance_profile.name,
                # Consul relies on EC2 metadata
                metadata_options=aws.ec2.InstanceMetadataOptionsArgs(
                    http_endpoint="enabled",
                    # There's a chance we can change this to "required", which forces IMDSv2,
                    # but we haven't tested that yet.
                    http_tokens="optional",
                ),
                opts=child_opts,
            )

            instances.append(instance)
        return instances
