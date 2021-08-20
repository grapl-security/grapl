import unittest
from typing import Optional, Tuple

import pulumi


class MyMocks(pulumi.runtime.Mocks):
    def new_resource(
        self, args: pulumi.runtime.MockResourceArgs
    ) -> Tuple[Optional[str], dict]:
        outputs = args.inputs
        return args.name + "_id", outputs

    # Officially call() should return Tuple[dict, Optional[List[Tuple[str,str]]]]. However there's a bug where when a
    # dict is not returned, call doesn't work as expected
    def call(self, args: pulumi.runtime.MockCallArgs) -> dict:  # type: ignore
        if args.token == "aws:index/getAvailabilityZones:getAvailabilityZones":
            return {"names": ["a", "b", "c"], "state": "available"}
        # This is necessary since infra.config calls this
        if args.token == "aws:index/getCallerIdentity:getCallerIdentity":
            return {"account_id": "000000000000"}
        return {}


pulumi.runtime.set_mocks(MyMocks())

# mocks must be defined prior to importing pulumi resources
from infra.network import Network


class TestingNetwork(unittest.TestCase):
    @pulumi.runtime.test
    def test_number_of_subnets(self) -> pulumi.Output:
        network = Network("test_public_subnets")

        def check_number_of_subnets(args: list) -> None:
            public_subnets, private_subnets = args
            assert (
                len(public_subnets) == 2
            ), "There should be 2 public subnets since desired_az_spread is 2"
            assert len(public_subnets) == len(
                private_subnets
            ), "There should be the same number of public subnets and private subnets"

        return pulumi.Output.all(network.public_subnets, network.private_subnets).apply(
            check_number_of_subnets
        )

    @pulumi.runtime.test
    def test_public_subnet_tags(self) -> pulumi.Output:
        network = Network("test_public_subnet_tags")

        def check_public_subnet_tags(args: list) -> None:
            urn, tags = args
            assert tags, f"Subnet {urn} must have tags"
            assert "Name" in tags, f"Subnet {urn} must have a Name tag"

        return pulumi.Output.all(
            network.public_subnets[0].urn, network.public_subnets[0].tags
        ).apply(check_public_subnet_tags)

    @pulumi.runtime.test
    def test_private_subnet_tags(self) -> pulumi.Output:
        network = Network("test_private_subnet_tags")

        def check_private_subnet_tags(args: list) -> None:
            urn, tags = args
            assert tags, f"Subnet {urn} must have tags"
            assert "Name" in tags, f"Subnet {urn} must have a Name tag"

        return pulumi.Output.all(
            network.private_subnets[0].urn, network.public_subnets[0].tags
        ).apply(check_private_subnet_tags)
