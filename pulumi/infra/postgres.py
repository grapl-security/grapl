from __future__ import annotations

from dataclasses import dataclass
from typing import List, Optional

import pulumi_aws as aws
import pulumi_random as random

import pulumi


@dataclass
class PostgresConfigValues:
    instance_type: str
    postgres_version: str

    @staticmethod
    def from_config() -> PostgresConfigValues:
        return PostgresConfigValues(
            instance_type=pulumi.Config().require("postgres-instance-type"),
            postgres_version=pulumi.Config().require("postgres-version"),
        )


class Postgres(pulumi.ComponentResource):
    """
    A Postgres instance running in RDS.
    """

    def __init__(
        self,
        name: str,
        vpc_id: pulumi.Input[str],
        subnet_ids: pulumi.Input[List[str]],
        nomad_agent_security_group_id: pulumi.Input[str],
        availability_zone: pulumi.Input[str],
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:Postgres", name, None, opts)

        child_opts = pulumi.ResourceOptions(parent=self)

        username = "postgres"

        # FYI we ran into some weird pulumi bugs around RandomPassword.
        # https://grapl-internal.slack.com/archives/C0174A8QV2S/p1642183461098100
        password = random.RandomPassword(
            f"{name}-password",
            length=32,
            # Disable special characters, ":" can lead to unexpected
            # sqlx errors: https://github.com/launchbadge/sqlx/issues/1624
            special=False,
            opts=pulumi.ResourceOptions(parent=self),
        )

        subnet_group_name = f"{name}-postgres-subnet-group"
        rds_subnet_group = aws.rds.SubnetGroup(
            subnet_group_name,
            subnet_ids=subnet_ids,
            tags={"Name": subnet_group_name},
            opts=child_opts,
        )

        sg_name = f"{name}-postgres-security-group"
        self.security_group = aws.ec2.SecurityGroup(
            sg_name,
            vpc_id=vpc_id,
            # Tags are necessary for the moment so we can look up the resource from a different pulumi stack.
            # Once this is refactored we can remove the tags
            tags={"Name": sg_name},
            opts=pulumi.ResourceOptions(parent=self),
        )
        postgres_port = 5432

        # Allow communication between nomad-agents and RDS
        aws.ec2.SecurityGroupRule(
            f"{name}-nomad-agents-egress-to-rds",
            type="egress",
            security_group_id=nomad_agent_security_group_id,
            from_port=postgres_port,
            to_port=postgres_port,
            protocol="tcp",
            source_security_group_id=self.security_group.id,
            opts=pulumi.ResourceOptions(parent=self.security_group),
        )

        aws.ec2.SecurityGroupRule(
            f"{name}-rds-ingress-from-nomad-agents",
            type="ingress",
            security_group_id=self.security_group.id,
            from_port=postgres_port,
            to_port=postgres_port,
            protocol="tcp",
            source_security_group_id=nomad_agent_security_group_id,
            opts=pulumi.ResourceOptions(parent=self.security_group),
        )

        postgres_config = PostgresConfigValues.from_config()

        self.instance = aws.rds.Instance(
            f"{name}-instance",
            name=name,  # only alphanumeric
            engine="postgres",
            engine_version=postgres_config.postgres_version,
            instance_class=postgres_config.instance_type,
            # These storage parameters should be more thoroughly thought out.
            allocated_storage=10,
            max_allocated_storage=20,
            # Subnet/vpc stuff
            db_subnet_group_name=rds_subnet_group.id,
            vpc_security_group_ids=[self.security_group.id],
            availability_zone=availability_zone,
            # TODO: EnableIAMDatabaseAuthentication
            # Makes teardown easier. This can be revisited.
            skip_final_snapshot=True,
            # Eventually this would be managed by Vault.
            # In the mean time, security is basically only enforced by VPC
            username=username,
            password=password.result,
            port=postgres_port,
            opts=child_opts,
        )
