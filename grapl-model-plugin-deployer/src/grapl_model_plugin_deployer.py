import json
import hmac
from hashlib import sha1

import boto3
from botocore.client import BaseClient
from github import Github

from base64 import b64decode

import os


def verify_payload(payload_body, key, signature):
    new_signature = "sha1=" + hmac.new(key, payload_body, sha1).hexdigest()
    return new_signature == signature


def lambda_handler(event, context):
    body = json.loads(event["body"])

    shared_secret = os.environ["GITHUB_SHARED_SECRET"]
    access_token = os.environ["GITHUB_ACCESS_TOKEN"]

    signature = event["headers"]["X-Hub-Signature"]

    assert verify_payload(
        event["body"].encode("utf8"), shared_secret.encode(), signature
    )

    repo_name = body["repository"]["full_name"]
    if body["ref"] != "refs/heads/master":
        return

    g = Github(access_token)
    s3_client = boto3.client("s3")

    repo = g.get_repo(repo_name)
    print(repo.name)

    analyzer_folders = repo.get_contents("plugins")
    # Upload every single file and folder, within 'plugins', to Grapl
    for plugin_folder in plugin_folders:
        pass


    # By convention, every schema file will hold one or many NodeSchema's,
    # which we will load and deploy to DGraph

    # TODO: Any forward edges created by plugins should, in turn, generate reverse edges in other types

    # Upload all schemas


def upload_analyzer(s3_client: BaseClient, name: str, contents: str) -> None:
    analyzer_bucket = os.environ["BUCKET_PREFIX"] + "-model-plugins-bucket"

    s3_client.put_object(
        Body=contents, Bucket=analyzer_bucket, Key=f"analyzers/{name}/main.py"
    )