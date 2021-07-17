import itertools
from typing import Dict, List, Optional, Sequence, Union

import pulumi_aws as aws
from infra.network import Network
from typing_extensions import Literal

import pulumi
from pulumi import Output

# Note that a Quorum of 1 will lose data across updates, not suitable
# for databases but fine for stateless services
QuorumSize = Union[Literal[1], Literal[3], Literal[5]]


class Ec2Cluster(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        vpc: Network,
        quorum_size: QuorumSize,
        quorums: int,  # Number of quorums to actually build
        ami: str,
        instance_type: str,
        iam_instance_profile: aws.iam.InstanceProfile,
        vpc_security_group_ids: Sequence[Output[str]],
        instance_tags: Dict[str, str],
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:Ec2ClusterResource", name=name, props=None, opts=opts)
        child_opts = pulumi.ResourceOptions(parent=self)

        self.instances = []
        for i in range(0, quorums):
            self.instances.extend(
                self._build_quorum(
                    f"{name}-quorum-{i}",
                    vpc,
                    ami,
                    instance_type,
                    quorum_size,
                    iam_instance_profile,
                    vpc_security_group_ids,
                    instance_tags,
                    child_opts,
                )
            )
        self.register_outputs(
            {
                "instances": self.instances,
            }
        )

    @staticmethod
    def _build_quorum(
        name: str,
        vpc: Network,
        ami: str,
        instance_type: str,
        quorum_size: QuorumSize,
        iam_instance_profile: aws.iam.InstanceProfile,
        vpc_security_group_ids: Sequence[Output[str]],
        instance_tags: Dict[str, str],
        child_opts: pulumi.ResourceOptions,
    ) -> List[aws.ec2.Instance]:
        instances = []
        # We're going to create each instance in a different private subnet. This way
        # our quorum will be resilient to AZ failures so long as there are quorum_size - 1 nodes
        # are still available

        _subnets = vpc.private_subnets
        subnets = itertools.cycle(_subnets)
        for i in range(0, quorum_size):
            print(f"name: {name}-{i}")
            subnet = next(subnets)
            network_interface = aws.ec2.NetworkInterface(
                f"ec2-eni-{name}-{i}",
                subnet_id=subnet.id,
                security_groups=vpc_security_group_ids,
                tags={
                    "Name": "primary_network_interface",
                },
                opts=child_opts,
            )
            instance = aws.ec2.Instance(
                f"ec2-inst-{name}-{i}",
                ami=ami,
                instance_type=instance_type,
                network_interfaces=[
                    aws.ec2.InstanceNetworkInterfaceArgs(
                        network_interface_id=network_interface.id,
                        device_index=0,
                    )
                ],
                credit_specification=aws.ec2.InstanceCreditSpecificationArgs(
                    cpu_credits="unlimited",
                ),
                tags=instance_tags,
                iam_instance_profile=iam_instance_profile.name,
                # vpc_security_group_ids=vpc_security_group_ids,
                metadata_options=aws.ec2.InstanceMetadataOptionsArgs(  # Consul relies on EC2 metadata
                    http_endpoint="enabled",
                    http_tokens="optional",  # Can we at least use v2? Should verify.
                ),
                opts=child_opts,
            )

            instances.append(instance)
        return instances
