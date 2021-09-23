import ipaddress
import math
from typing import Optional

import pulumi_aws as aws
from infra.config import DEPLOYMENT_NAME

import pulumi

# TODO: Add policy requiring that each networking-related
# infrastructure component have a "Name" (note capitalization) tag,
# since this is how these objects get their names. Otherwise, they
# have no name, which makes interaction via the AWS UI slightly more
# awkward than it needs to be.
#
# Security groups don't seem to work this way, though. They get a name
# independent of a Name tag (though the AWS UI shows both, somewhat
# confusingly).


class Network(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:Network", name, None, opts)

        # From CDK's @aws-cdk/aws-ec2.Vpc Construct (which we used
        # before):
        #
        # It will automatically divide the provided VPC CIDR range,
        # and create public and private subnets per Availability
        # Zone. Network routing for the public subnets will be
        # configured to allow outbound access directly via an Internet
        # Gateway. Network routing for the private subnets will be
        # configured to allow outbound access via a set of resilient
        # NAT Gateways (one per AZ).

        # TODO: Is it worth setting up routing tables, network ACLs
        # here, too? Currently, the default ones are all we need, but
        # it could be useful in the future to have Pulumi be aware of
        # them.

        cidr_block = ipaddress.ip_network("10.0.0.0/16")

        self.vpc = aws.ec2.Vpc(
            f"{name}-vpc",
            cidr_block=str(cidr_block),
            enable_dns_hostnames=True,
            enable_dns_support=True,
            tags={"Name": f"{name}"},
            opts=pulumi.ResourceOptions(parent=self),
        )

        # We will create a public and a private subnet per
        # availability zone for some number of zones. Our CDK code
        # defaulted to 2 AZs, so that's what we'll do here.
        azs = aws.get_availability_zones(state="available").names

        # Split the block into two non-overlapping subnets; these will
        # be further divided by availability zone
        public, private = cidr_block.subnets()

        # Always assume at least 2 availibity zones; no region has
        # less. us-east-1 currently has the most AZs at 6.
        #
        # TODO: Consider parameterizing this value. This just happens
        # to be what we ended up with from our CDK usage. Also
        # consider implications of scaling up/down with the
        # delete/recreate semantics of subnets (which can't overlap).
        desired_az_spread = 2
        num_azs = min(desired_az_spread, len(azs))
        # Once desired_az_spread is parameterized the below will be true. Until then it will always be 2 or fewer.
        # 2 AZs => 2 subnets
        # 3 AZs => 4 subnets (extra 1 doesn't actually get created)
        # 4 AZs => 4 subnets
        # 5 AZs => 8 subnets (extra 3 don't actually get created)
        # etc.
        prefixlen_diff = math.ceil(math.log(num_azs, 2))

        self.public_subnets = [
            aws.ec2.Subnet(
                f"{name}-{az}-public-subnet",
                vpc_id=self.vpc.id,
                availability_zone=az,
                cidr_block=str(subnet),
                map_public_ip_on_launch=True,
                tags={"Name": f"{name}-{DEPLOYMENT_NAME}-{az}-public-subnet"},
                opts=pulumi.ResourceOptions(parent=self),
            )
            for az, subnet in zip(
                azs[:num_azs], public.subnets(prefixlen_diff=prefixlen_diff)
            )
        ]

        self.private_subnets = [
            aws.ec2.Subnet(
                f"{name}-{az}-private-subnet",
                vpc_id=self.vpc.id,
                availability_zone=az,
                cidr_block=str(subnet),
                map_public_ip_on_launch=False,
                tags={
                    "Name": f"{name}-{DEPLOYMENT_NAME}-{az}-private-subnet",
                    # Consumed by `grapl_subnet_ids` in graplctl
                    "graplctl_get_subnet_tag": f"{DEPLOYMENT_NAME}-private-subnet",
                },
                opts=pulumi.ResourceOptions(parent=self),
            )
            for az, subnet in zip(
                azs[:num_azs], private.subnets(prefixlen_diff=prefixlen_diff)
            )
        ]

        self.internet_gateway = aws.ec2.InternetGateway(
            f"{name}-internet-gateway",
            vpc_id=self.vpc.id,
            tags={
                "Name": f"{DEPLOYMENT_NAME}-internet-gateway",
            },
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.eip = aws.ec2.Eip(
            f"{name}-eip",
            vpc=True,
            opts=pulumi.ResourceOptions(parent=self),
        )

        # TODO: Do we want to manage the NICs as well?

        # Note that NAT gateways need to have an internet gateway
        # attached to the VPC first... that's why we have a depends_on
        # value set.
        self.nat_gateway = aws.ec2.NatGateway(
            f"{name}-nat-gateway",
            subnet_id=self.public_subnets[0].id,
            # This is id instead of allocation_id because of a bug
            # https://github.com/pulumi/pulumi-aws/issues/498
            allocation_id=self.eip.id,
            tags={
                "Name": f"{DEPLOYMENT_NAME}-nat-gateway",
            },
            opts=pulumi.ResourceOptions(
                parent=self, depends_on=[self.internet_gateway]
            ),
        )

        for subnet in self.public_subnets:
            name = f"route-for-{subnet._name}"
            route_table = aws.ec2.RouteTable(
                name,
                vpc_id=self.vpc.id,
                routes=[
                    aws.ec2.RouteTableRouteArgs(
                        cidr_block="0.0.0.0/0",
                        gateway_id=self.internet_gateway.id,
                    )
                ],
                tags={"Name": name},
                opts=pulumi.ResourceOptions(parent=subnet),
            )

            aws.ec2.RouteTableAssociation(
                f"assoc-{subnet._name}",
                subnet_id=subnet.id,
                route_table_id=route_table.id,
                opts=pulumi.ResourceOptions(parent=subnet),
            )

        for subnet in self.private_subnets:
            name = f"route-for-{subnet._name}"
            route_table = aws.ec2.RouteTable(
                name,
                vpc_id=self.vpc.id,
                routes=[
                    aws.ec2.RouteTableRouteArgs(
                        cidr_block="0.0.0.0/0",
                        nat_gateway_id=self.nat_gateway.id,
                    )
                ],
                tags={"Name": name},
                opts=pulumi.ResourceOptions(parent=subnet),
            )

            aws.ec2.RouteTableAssociation(
                f"assoc-{subnet._name}",
                subnet_id=subnet.id,
                route_table_id=route_table.id,
                opts=pulumi.ResourceOptions(parent=subnet),
            )

        self.register_outputs({})
