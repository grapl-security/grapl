import json
from typing import Optional

import pulumi_aws as aws
from infra.bucket import Bucket
from infra.config import DEPLOYMENT_NAME
from infra.dgraph_cluster import DgraphCluster
from infra.dynamodb import DynamoDB
from infra.network import Network

import pulumi


class EngagementNotebook(pulumi.ComponentResource):
    def __init__(
        self,
        network: Network,
        db: DynamoDB,
        plugins_bucket: Bucket,
        dgraph_cluster: DgraphCluster,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:

        # TODO: Should this really have its own name? What should the
        # criteria be for giving things names?
        name = "engagement-notebook"

        super().__init__("grapl:EngagementNotebook", name, None, opts)

        self.security_group = aws.ec2.SecurityGroup(
            f"{name}-security-group",
            vpc_id=network.vpc.id,
            egress=[
                aws.ec2.SecurityGroupEgressArgs(
                    description="Allow all outgoing connections",
                    protocol="-1",  # semantically "all"
                    from_port=0,
                    to_port=0,
                    cidr_blocks=["0.0.0.0/0"],
                )
            ],
            opts=pulumi.ResourceOptions(parent=self),
        )
        dgraph_cluster.allow_connections_from(self.security_group)

        # TODO: Allow connections to DGraph

        # TODO: Consider creating a base role class... pass in name,
        # description, principal, optional managed arns, and opts
        self.role = aws.iam.Role(
            f"{name}-role",
            description="Engagement notebook role",
            assume_role_policy=json.dumps(
                {
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Effect": "Allow",
                            "Action": "sts:AssumeRole",
                            "Principal": {"Service": "sagemaker.amazonaws.com"},
                        }
                    ],
                }
            ),
            opts=pulumi.ResourceOptions(parent=self),
        )

        db.user_auth_table.grant_read_write_permissions_to(self.role)
        db.schema_table.grant_read_write_permissions_to(self.role)
        plugins_bucket.grant_read_permissions_to(self.role)

        self.notebook = aws.sagemaker.NotebookInstance(
            f"{name}-instance",
            # TODO: Remove this name eventually; currently
            # engagement-edge relies on it being set like this.
            # Remove delete_before_replace below once we make this change.
            name=f"{DEPLOYMENT_NAME}-Notebook",
            instance_type="ml.t2.medium",
            role_arn=self.role.arn,
            security_groups=[self.security_group.id],
            subnet_id=network.private_subnets[0].id,
            direct_internet_access="Enabled",  # TODO: this is the default... needed?
            # Only using delete_before_replace while we are specifying
            # a concrete physical name
            opts=pulumi.ResourceOptions(parent=self, delete_before_replace=True),
        )

        self.register_outputs({})

    # See https://github.com/grapl-security/issue-tracker/issues/115 for details.
    def grant_presigned_url_permissions_to(self, role: aws.iam.Role) -> None:
        aws.iam.RolePolicy(
            f"{role._name}-creates-presigned-urls-for-{self.notebook._name}",
            role=role.name,
            policy=self.notebook.arn.apply(
                lambda notebook_arn: json.dumps(
                    {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Effect": "Allow",
                                "Action": "sagemaker:CreatePresignedNotebookInstanceUrl",
                                "Resource": notebook_arn,
                            }
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=self),
        )
