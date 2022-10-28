from __future__ import annotations

import os

import boto3
from grapl_common.env_helpers import SecretsManagerClientFactory
from grapl_common.logger import get_structlogger

LOGGER = get_structlogger()


def get_test_user_creds() -> tuple[str, str]:
    username = os.environ["GRAPL_TEST_USER_NAME"]
    password_secret_id = os.environ["GRAPL_TEST_USER_PASSWORD_SECRET_ID"]
    LOGGER.debug(f"Retrieving secret id: {password_secret_id}")
    secrets_client = SecretsManagerClientFactory(boto3).from_env()
    password = secrets_client.get_secret_value(SecretId=password_secret_id)[
        "SecretString"
    ]
    return (username, password)
