from typing import List, Optional, Union

import pulumi
import pulumi_aws as aws
import pulumi_random as random

from infra.config import DEPLOYMENT_NAME


class RdsInstance(pulumi.ComponentResource):
    def __init__(
            self,
            name: str,
            subnet_ids: pulumi.Input[List[str]],
            vpc_id: pulumi.Input[str],
            username: str = "admin",
            port: int = 5432,
            identifier_prefix: str = None,
            storage_type: Union[str, aws.rds.StorageType] = 'gp2',
            allocated_storage: Optional[int] = None,
            max_allocated_storage: Optional[int] = None,
            allow_major_version_upgrade: Optional[bool] = None,
            auto_minor_version_upgrade: Optional[bool] = None,
            backup_retention_period: Optional[int] = None,
            opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:PostgresInstance", name, None, opts)

        # Note that this password is not a control. We control access
        # to RDS through IAM. This password will likely end up in logs.
        password = random.RandomPassword(f'{name}-sng',
                                         length=24,
                                         special=False,
                                         )
        self.password = password.result

        # self.rds_subnet_group = aws.rds.SubnetGroup(f'{name}-sng',
        #                                             subnet_ids=subnet_ids,
        #                                             tags={"Name": f"{name}-{DEPLOYMENT_NAME}"},
        #                                             opts=pulumi.ResourceOptions(parent=self)
        #                                             )

        # self.security_group = aws.ec2.SecurityGroup(
        #     f"{name}-cache-security-group",
        #     vpc_id=vpc_id,
        #     # Tags are necessary for the moment so we can look up the resource from a different pulumi stack.
        #     # Once this is refactored we can remove the tags
        #     tags={"Name": f"{name}-{DEPLOYMENT_NAME}"},
        #     opts=pulumi.ResourceOptions(parent=self),
        # )

        self.instance = aws.rds.Instance(
            name,
            engine="postgres",
            username=username,
            password=self.password,
            port=port,
            opts=opts,
            # vpc_security_group_ids=[self.security_group.id],
            # db_subnet_group_name=None,  # Temporary
            allocated_storage=allocated_storage,
            max_allocated_storage=max_allocated_storage,
            allow_major_version_upgrade=allow_major_version_upgrade,
            auto_minor_version_upgrade=auto_minor_version_upgrade,
            backup_retention_period=backup_retention_period,
            identifier_prefix=identifier_prefix,
            storage_type=storage_type
        )
