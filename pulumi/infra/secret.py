import json
from typing import Dict, Optional

import pulumi_aws as aws
import pulumi_random as random
from infra.config import DEPLOYMENT_NAME, LOCAL_GRAPL

import pulumi


class _Secret(pulumi.ComponentResource):
    def __init__(
        self,
        t: str,
        name: str,
        secret: aws.secretsmanager.Secret,
        props: Optional[Dict] = None,
        opts: Optional[pulumi.ResourceOptions] = None,
        remote: bool = False,
    ) -> None:
        super().__init__(t, name, props, opts, remote)
        self.secret = secret

    def grant_read_permissions_to(self, role: aws.iam.Role) -> None:
        """
        Grants permission to the given `Role` to read this secret.

        The name of the resource is formed from the Pulumi name of the `Role`.
        """
        aws.iam.RolePolicy(
            f"{role._name}-reads-secret",
            role=role.name,
            policy=self.secret.arn.apply(
                lambda secret_arn: json.dumps(
                    {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Effect": "Allow",
                                "Action": [
                                    "secretsmanager:GetSecretValue",
                                    "secretsmanager:DescribeSecret",
                                ],
                                "Resource": secret_arn,
                            },
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=self),
        )


class JWTSecret(_Secret):
    """ Represents the frontend's JWT secret stored in Secretsmanager. """

    def __init__(self, opts: Optional[pulumi.ResourceOptions] = None) -> None:
        super().__init__(
            "grapl:JWTSecret",
            "jwt-secret",
            aws.secretsmanager.Secret(
                "edge-jwt-secret",
                # TODO: Ultimately we don't want to care about this... it's
                # just what the local services expect at the moment. As we
                # move more things over to Pulumi, we'll be able to inject
                # this automatically into, e.g., Lambda function environments.
                name="JWT_SECRET_ID" if LOCAL_GRAPL else None,
                description="The JWT secret that Grapl uses to authenticate its API",
                opts=pulumi.ResourceOptions(parent=self),
            ),
            None,
            opts
        )

        self.random_uuid = random.RandomUuid(
            "jwt-secret-uuid",
            opts=pulumi.ResourceOptions(
                parent=self, additional_secret_outputs=["result"]
            ),
        )

        # TODO: What do we do about rotation?
        self.version = aws.secretsmanager.SecretVersion(
            "secret",
            secret_id=self.secret.id,
            secret_string=self.random_uuid.result,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.register_outputs({})


class TestUserPassword(_Secret):
    """ Grapl password for the test user. """

    def __init__(self, opts: Optional[pulumi.ResourceOptions] = None) -> None:
        super().__init__(
            "grapl:TestUserPassword",
            "test-user-password",
            aws.secretsmanager.Secret(
                "test-user-password",
                name=f"{DEPLOYMENT_NAME}-TestUserPassword",
                description="The Grapl test user's password",
                opts=pulumi.ResourceOptions(parent=self),
            ),
            None,
            opts
        )

        self.random_uuid = random.RandomUuid(
            "test-user-password-string",
            opts=pulumi.ResourceOptions(
                parent=self, additional_secret_outputs=["result"]
            ),
            opts=pulumi.ResourceOptions(parent=role),
        )

        # TODO: What do we do about rotation?
        self.version = aws.secretsmanager.SecretVersion(
            "secret",
            secret_id=self.secret.id,
            secret_string=self.random_uuid.result,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.register_outputs({})
