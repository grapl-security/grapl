import json
from typing import Dict

import pulumi_aws as aws
from argon2 import PasswordHasher

import pulumi


def local_user_item(username: str, cleartext: str) -> Dict[str, Dict[str, str]]:
    """
    Creates an "owner" account with the given username and password (cleartext)
    """

    password_hasher = PasswordHasher(time_cost=2, memory_cost=102400, parallelism=8)
    password_hash = password_hasher.hash(cleartext)

    return {
        "username": {"S": username},
        "password_hash": {"S": password_hash},
        "role": {"S": "owner"},
    }


def local_grapl_user(table: aws.dynamodb.Table, username: str, cleartext: str) -> None:
    """Create a user only for local development uses; NEVER REAL AWS"""

    deployment_name = pulumi.get_stack()

    user = aws.dynamodb.TableItem(
        f"{deployment_name}-user-{username}",
        table_name=table.name,
        hash_key=table.hash_key,
        item=json.dumps(local_user_item(username, cleartext)),
    )

    pulumi.export(f"user-{username}", user.id)
