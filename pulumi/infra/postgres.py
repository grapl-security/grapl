from typing import List, Optional

import pulumi_aws as aws
import pulumi_random as random

import pulumi


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
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:Postgres", name, None, opts)

        # Burstable, 2GB memory, Gravitron 2.
        # With 5GB of storage and all-day usage, it comes out to about $24/mo
        instance_class = "db.t4g.small"

        child_opts = pulumi.ResourceOptions(parent=self)

        self.username = "postgres"
        self.password = random.RandomPassword(
            "password",
            length=32,
            # Disable special characters, ":" can lead to unexpected
            # sqlx errors: https://github.com/launchbadge/sqlx/issues/1624
            special=False,
            opts=child_opts,
        ).result

        rds_subnet_group = aws.rds.SubnetGroup(
            "subnet-group",
            subnet_ids=subnet_ids,
            tags={"Name": f"{name}-subnet-group"},
            opts=child_opts,
        )

        self.security_group = aws.ec2.SecurityGroup(
            f"postgres-security-group",
            vpc_id=vpc_id,
            # Tags are necessary for the moment so we can look up the resource from a different pulumi stack.
            # Once this is refactored we can remove the tags
            tags={"Name": f"{name}-postgres-security-group"},
            opts=pulumi.ResourceOptions(parent=self),
        )
        postgres_port = 5432

        # Allow communication between nomad-agents and RDS
        aws.ec2.SecurityGroupRule(
            "nomad-agents-egress-to-rds",
            type="egress",
            security_group_id=nomad_agent_security_group_id,
            from_port=postgres_port,
            to_port=postgres_port,
            protocol="tcp",
            source_security_group_id=self.security_group.id,
            opts=pulumi.ResourceOptions(parent=self.security_group),
        )

        aws.ec2.SecurityGroupRule(
            "rds-ingress-from-nomad-agents",
            type="ingress",
            security_group_id=self.security_group.id,
            from_port=postgres_port,
            to_port=postgres_port,
            protocol="tcp",
            source_security_group_id=nomad_agent_security_group_id,
            opts=pulumi.ResourceOptions(parent=self.security_group),
        )

        # There's some funkiness where, if not specified, you may end up with
        # RDS trying to stand up an instance in an AZ not included in `subnets`.
        def to_az(ids: List[str]) -> pulumi.Output[str]:
            subnet_id = ids[0]
            subnet = aws.ec2.Subnet.get("subnet", subnet_id)
            # for some reason mypy gets hung up on the typing of this
            az: pulumi.Output[str] = subnet.availability_zone
            return az

        availability_zone: pulumi.Output[str] = pulumi.Output.from_input(
            subnet_ids
        ).apply(to_az)

        self.instance = aws.rds.Instance(
            "instance",
            name=name,  # only alphanumeric
            engine="postgres",
            engine_version="13.4",
            instance_class=instance_class,
            allocated_storage=10,
            max_allocated_storage=20,
            availability_zone=availability_zone,
            # Subnet/vpc stuff
            db_subnet_group_name=rds_subnet_group.id,
            vpc_security_group_ids=[self.security_group.id],
            # TODO: EnableIAMDatabaseAuthentication
            # Makes teardown easier. This can be revisited.
            skip_final_snapshot=True,
            # Eventually this would be managed by Vault.
            # In the mean time, security is basically only enforced by VPC
            username=self.username,
            password=self.password,
            port=postgres_port,
            # Note: See Pulumi docs about `apply_immediately` flag for ASAP migrations
            opts=child_opts,
        )
