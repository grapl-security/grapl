import pulumi
import sys
import unittest
from unittest.mock import MagicMock, patch


class MyMocks(pulumi.runtime.Mocks):
    def new_resource(self, args: pulumi.runtime.MockResourceArgs):
        outputs = args.inputs
        return [args.name + '_id', outputs]

    def call(self, args: pulumi.runtime.MockCallArgs):
        if args.token == "aws:index/getAvailabilityZones:getAvailabilityZones":
            return {
                "names": ["a", "b", "c"],
                "state": "available"
            }
        # This is necessary since infra.config calls this
        if args.token == "aws:index/getCallerIdentity:getCallerIdentity":
            return {
                "account_id": "000000000000"
            }
        return {}

pulumi.runtime.set_mocks(MyMocks())

sys.modules['infra.network.infra.config'] = MagicMock()

# mocks must be defined prior to importing pulumi resources
from infra.network import Network


@patch('infra.config.DEPLOYMENT_NAME', 'unit-test-grapl')
class TestingNetwork(unittest.TestCase):
    @pulumi.runtime.test
    def test_number_of_subnets(self):
        network = Network("test_public_subnets")

        def check_number_of_subnets(args):
            public_subnets, private_subnets = args
            self.assertEqual(len(public_subnets), 2, f'There should be 2 public subnets since desired_az_spread is 2')
            self.assertEqual(len(public_subnets), len(private_subnets),
                             f'There should be the same number of public subnets and private subnets')

        return pulumi.Output.all(network.public_subnets, network.private_subnets).apply(check_number_of_subnets)

    @pulumi.runtime.test
    def test_public_subnet_tags(self):
        network = Network("test_public_subnet_tags")

        def check_public_subnet_tags(args):
            urn, tags = args
            self.assertIsNotNone(tags, f'Subnet {urn} must have tags')
            self.assertIn("Name", tags, f'Subnet {urn} must have a Name tag')

        return pulumi.Output.all(network.public_subnets[0].urn, network.public_subnets[0].tags).apply(check_public_subnet_tags)

    @pulumi.runtime.test
    def test_private_subnet_tags(self):
        network = Network("test_private_subnet_tags")

        def check_private_subnet_tags(args):
            urn, tags = args
            self.assertIsNotNone(tags, f'Subnet {urn} must have tags')
            self.assertIn("Name", tags, f'Subnet {urn} must have a Name tag')

        return pulumi.Output.all(network.private_subnets[0].urn, network.public_subnets[0].tags).apply(check_private_subnet_tags)
