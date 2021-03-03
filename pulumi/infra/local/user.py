import json
import os
from base64 import b64encode
from hashlib import pbkdf2_hmac, sha256
from typing import Dict

import pulumi_aws as aws

import pulumi


def local_user_item(username: str, cleartext: str) -> Dict[str, Dict[str, str]]:
    # We hash before calling 'hashed_password' because the frontend will also perform
    # client side hashing
    cleartext += "f1dafbdcab924862a198deaa5b6bae29aef7f2a442f841da975f1c515529d254"
    cleartext += username

    hashed = sha256(cleartext.encode("utf8")).hexdigest()
    for i in range(0, 5000):
        hashed = sha256(hashed.encode("utf8")).hexdigest()

    salt = os.urandom(16)
    password = hash_password(hashed.encode("utf8"), salt)

    return {
        "username": {"S": username},
        "salt": {"B": str(b64encode(salt), "utf-8")},
        "password": {"S": password},
    }


def hash_password(cleartext: bytes, salt: bytes) -> str:
    hashed = sha256(cleartext).digest()
    return pbkdf2_hmac("sha256", hashed, salt, 512000).hex()


def local_grapl_user(table: aws.dynamodb.Table, username: str, cleartext: str) -> None:
    """Create a user only for local development uses; NEVER REAL AWS"""

    prefix = pulumi.get_stack()

    user = aws.dynamodb.TableItem(
        f"{prefix}-user-{username}",
        table_name=table.name,
        hash_key=table.hash_key,
        item=json.dumps(local_user_item(username, cleartext)),
    )

    pulumi.export(f"user-{username}", user.id)
