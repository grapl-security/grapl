from __future__ import annotations

import re
from dataclasses import dataclass
from typing import List, Optional, cast

import pulumi_aws as aws
import pulumi_random as random
from infra.nomad_service_postgres import NomadServicePostgresDbArgs
from packaging.version import parse as version_parse

import pulumi


@dataclass
class PostgresConfigValues:
    instance_type: str
    postgres_version: str

    def __post_init__(self) -> None:
        # postgres uses 2-part semver
        assert version_parse(self.postgres_version) >= version_parse(
            "13.4"
        ), "Version must be >= 13.4"

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

        # Parameter Groups are what we use to preload pg libraries
        parameter_group = aws.rds.ParameterGroup(
            name,
            description=f"{name} managed by Pulumi",
            # TODO autoparse the family
            family="postgres13",
            parameters=[
                {
                    "name": "shared_preload_libraries",
                    "value": "pg_cron,pg_stat_statements",
                    "apply_method": "pending-reboot",
                }
            ],
        )

        postgres_config = PostgresConfigValues.from_config()

        # Quick diatribe:
        # "The database name is the name of a database hosted in your DB instance.
        #  A database name is not required when creating a DB instance.
        #  Databases hosted by the same DB instance must have a unique name within
        #  that instance."
        # Since we're using 1 database per instance right now, I'm going to
        # hardcode it to the default value of `postgres` (like the postgres
        # docker image does).
        #
        # As for giving the instance a name we can see on the console, that's
        # the `identifier=`.
        database_name = "postgres"
        assert re.match(
            "^[a-zA-Z][a-zA-Z0-9]+$", database_name
        ), "Database name must be alpha+alphanumeric"

        instance_name = f"{name}-instance"
        self._instance = aws.rds.Instance(
            instance_name,
            identifier=instance_name,
            db_name=database_name,  # See above diatribe
            engine="postgres",
            engine_version=postgres_config.postgres_version,
            instance_class=postgres_config.instance_type,
            parameter_group_name=parameter_group.name,
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
            # In the meantime, security is basically only enforced by VPC
            username=username,
            password=password.result,
            port=postgres_port,
            # Because we're not specifying a kms key, this will use the default aws/rds key
            storage_encrypted=True,
            # Enable performance insights for 7 days, which is free
            performance_insights_enabled=True,
            performance_insights_retention_period=7,
            # We need delete before replace in order to encrypt instances. Once prod is up and running we may want to
            # remove this option to avoid accidental downtime.
            opts=pulumi.ResourceOptions(parent=self, delete_before_replace=True),
        )

    def to_nomad_service_db_args(self) -> pulumi.Output[NomadServicePostgresDbArgs]:
        return cast(
            pulumi.Output[NomadServicePostgresDbArgs],
            pulumi.Output.all(
                hostname=self._instance.address,
                port=self._instance.port,
                username=self._instance.username,
                password=self._instance.password,
            ),
        )
