from uuid import uuid4

import pulumi_aws as aws

import pulumi


def jwt_secret() -> None:
    """Set up a JWT secret for use in local environments"""

    secret = aws.secretsmanager.Secret("JWT_SECRET_ID", name="JWT_SECRET_ID")

    version = aws.secretsmanager.SecretVersion(
        "jwt_secret", secret_id=secret.id, secret_string=str(uuid4())
    )

    pulumi.export("JWT_SECRET", version.secret_string)
