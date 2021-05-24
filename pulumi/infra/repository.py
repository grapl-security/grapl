import base64
import json
from typing import Optional

import pulumi_aws as aws
import pulumi_docker as docker
from infra.config import AWS_ACCOUNT_ID, DEPLOYMENT_NAME

import pulumi


def registry_credentials() -> docker.ImageRegistry:
    """Provide login credentials for our (private) ECR registry.

    (Each AWS account has such a registry.)

    The token is the base64 encoded concatenation of
    <USERNAME>:<TEMPORARY_PASSWORD>, where <USERNAME> is always "AWS",
    and <TEMPORARY_PASSWORD> is valid for 12 hours.

    The Pulumi resources involved are ultimately used by Docker to log
    into the registry. Since Docker doesn't natively understand ECR
    access methods, we have to do a bit of manipulation to tease apart
    the token manually.

    (Note that we are *not* assuming the use of
    https://github.com/awslabs/amazon-ecr-credential-helper!)

    See the following for more details:
    - https://docs.aws.amazon.com/AmazonECR/latest/APIReference/API_GetAuthorizationToken.html
    - https://docs.aws.amazon.com/AmazonECR/latest/userguide/registry_auth.html
    - https://docs.aws.amazon.com/cli/latest/reference/ecr/get-login-password.html
    - https://docs.aws.amazon.com/cli/latest/reference/ecr/get-login.html
    """
    credentials = aws.ecr.get_credentials(registry_id=AWS_ACCOUNT_ID)
    decoded = base64.b64decode(credentials.authorization_token).decode()
    username, password = decoded.split(":")

    return docker.ImageRegistry(
        server=f"https://{AWS_ACCOUNT_ID}.dkr.ecr.{aws.get_region().name}.amazonaws.com",
        username=username,
        password=password,
    )


class Repository(pulumi.ComponentResource):
    """Storage for a single line of container images.

    The name of this repository (and thus the images within it) will
    be `<DEPLOYMENT_NAME>/<image_name>` (that is, the images will be
    namespaced by deployment).

    """

    def __init__(
        self,
        image_name: str,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:ECRRepository", image_name, None, opts)

        self.repository = aws.ecr.Repository(
            image_name,
            name=f"{DEPLOYMENT_NAME}/{image_name}",
            opts=pulumi.ResourceOptions(parent=self, delete_before_replace=True),
        )

        self.register_outputs({})

    @property
    def registry_qualified_name(self) -> pulumi.Output[str]:
        """
        The fully-qualified image name for this repository, not including tags, e.g.,

        <AWS_ACCOUNT_ID>.dkr.ecr.<AWS_REGION>.amazonaws.com/<DEPLOYMENT_NAME>/<NAME>
        """
        return self.repository.repository_url  # type: ignore[no-any-return]

    def grant_access_to(self, role: aws.iam.Role) -> None:
        """This is the policy that would need to be attached to Fargate
        execution roles (for services that pull images from this
        repository, of course)."""

        aws.iam.RolePolicy(
            f"{role._name}-accesses-{self.repository._name}",
            role=role.name,
            policy=self.repository.arn.apply(
                lambda arn: json.dumps(
                    {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Effect": "Allow",
                                "Action": [
                                    "ecr:BatchCheckLayerAvailability",
                                    "ecr:GetDownloadUrlForLayer",
                                    "ecr:BatchGetImage",
                                ],
                                "Resource": arn,
                            }
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=self.repository),
        )
