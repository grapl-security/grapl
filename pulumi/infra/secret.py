import json
from typing import Optional

import pulumi_aws as aws
import pulumi_random as random
from infra.config import LOCAL_GRAPL, STACK_NAME

import pulumi


class JWTSecret(pulumi.ComponentResource):
    """ Represents the frontend's JWT secret stored in Secretsmanager. """

    def __init__(self, opts: Optional[pulumi.ResourceOptions] = None) -> None:
        super().__init__(
            "grapl:JWTSecret",
            "jwt-secret",
            None,
            opts,
        )

        self.secret = aws.secretsmanager.Secret(
            "edge-jwt-secret",
            # TODO: Ultimately we don't want to care about this... it's
            # just what the local services expect at the moment. As we
            # move more things over to Pulumi, we'll be able to inject
            # this automatically into, e.g., Lambda function environments.
            name="JWT_SECRET_ID" if LOCAL_GRAPL else None,
            description="The JWT secret that Grapl uses to authenticate its API",
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.random_uuid = random.RandomUuid(
            "jwt-secret-uuid",
            opts=pulumi.ResourceOptions(
                parent=self, additional_secret_outputs=["result"]
            ),
        )

        # TODO: What do we do about rotation?
        self.version = aws.secretsmanager.SecretVersion(
            "jwt-secret-version",
            secret_id=self.secret.id,
            secret_string=self.random_uuid.result,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.register_outputs({})

    def grant_read_permissions_to(self, role: aws.iam.Role) -> None:
        """
        Grants permission to the given `Role` to read this secret.

        The name of the resource is formed from the Pulumi name of the `Role`.
        """
        aws.iam.RolePolicy(
            f"{role._name}-reads-jwt-secret",
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
            opts=pulumi.ResourceOptions(parent=role),
        )


class TestUserPassword(pulumi.ComponentResource):
    """ Grapl password for the test user. """

    def __init__(self, opts: Optional[pulumi.ResourceOptions] = None) -> None:
        super().__init__(
            "grapl:TestUserPassword",
            "test-user-password",
            None,
            opts,
        )

        self.secret = aws.secretsmanager.Secret(
            "test-user-password",
            name=f"{STACK_NAME}-TestUserPassword",
            description="The Grapl test user's password",
            recovery_window_in_days=0,  # delete immediately
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.random_uuid = random.RandomUuid(
            "test-user-password-string",
            opts=pulumi.ResourceOptions(
                parent=self, additional_secret_outputs=["result"]
            ),
        )

        # TODO: What do we do about rotation?
        self.version = aws.secretsmanager.SecretVersion(
            "test-user-password-version",
            secret_id=self.secret.id,
            secret_string=self.random_uuid.result,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.register_outputs({})

    def grant_read_permissions_to(self, role: aws.iam.Role) -> None:
        """
        Grants permission to the given `Role` to read this secret.

        The name of the resource is formed from the Pulumi name of the `Role`.
        """
        aws.iam.RolePolicy(
            f"{role._name}-reads-test-user-password",
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
            opts=pulumi.ResourceOptions(parent=role),
        )
