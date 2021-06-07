import itertools
from dataclasses import dataclass
from typing import Optional, Literal, Union, List, Sequence

import pulumi

import pulumi_aws as aws
from pulumi import Output

from infra.network import Network

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
            child_opts,
    ) -> List[aws.ec2.Instance]:
        instances = []
        # We're going to create each instance in a different private subnet. This way
        # our quorum will be resilient to AZ failures so long as there are quorum_size - 1 nodes
        # are still available

        # Ideally we'll perform this update in a rolling fashion, where for a given quorum
        # * We terminate an instance 'A'
        # * We bring up a new instance 'B'
        # * We migrate the network interface from 'A' to 'B'
        # Each step would be done serially (within a quorum)

        # Sorting the subnets is an attempt to avoid a situation where we move an EC2
        # instance from one subnet to another.

        _subnets = vpc.private_subnets
        subnets = itertools.cycle(_subnets)
        for i in range(0, quorum_size):
            subnet = next(subnets)
            eni_name = Output.concat('ec2-eni-', name, '-', subnet.id, '-', i)
            instance_name = Output.concat('ec2-inst-', name, '-', subnet.id, '-', i)
            network_interface = eni_name.apply(lambda eni_name: (aws.ec2.NetworkInterface(
                eni_name,
                subnet_id=subnet.id,
                tags={
                    "Name": "primary_network_interface",
                },
                opts=child_opts,
            )))
            instance = instance_name.apply(lambda instance_name: (aws.ec2.Instance(
                instance_name,
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
                iam_instance_profile=iam_instance_profile.name,
                vpc_security_group_ids=vpc_security_group_ids,
                # metadata_options=aws.ec2.InstanceMetadataOptions(  # Consul relies on metadata
                #     # http_tokens=True  # Can we at least use v2?
                # ),
                opts=child_opts
            )))

            instances.append(instance)
        return instances
