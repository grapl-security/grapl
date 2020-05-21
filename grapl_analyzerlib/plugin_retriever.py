import base64
import os.path
from pathlib import Path

import boto3


def load_plugins(bucket_prefix: str, s3=None):
    s3 = s3 or boto3.resource("s3")

    PluginRetriever(
        plugin_bucket=bucket_prefix + "-model-plugins-bucket",
        plugin_directory="./model_plugins/",
        s3_client=s3.meta.client,
    ).retrieve(overwrite=True)


def load_plugins_local():
    bucket_prefix = "local-grapl"
    s3 = boto3.resource(
        "s3",
        endpoint_url="http://s3:9000",
        aws_access_key_id="minioadmin",
        aws_secret_access_key="minioadmin",
    )

    load_plugins(bucket_prefix, s3)


class PluginRetriever(object):
    def __init__(self, plugin_bucket: str, plugin_directory: str, s3_client,) -> None:
        self.plugin_bucket = plugin_bucket
        self.s3_client = s3_client
        self.plugin_directory = plugin_directory

    def retrieve(self, overwrite: bool = False) -> None:
        # list plugin files
        plugin_objects = self.s3_client.list_objects(Bucket=self.plugin_bucket,).get(
            "Contents", []
        )

        # Download each one to the /plugins/ directory
        for plugin_object in plugin_objects:
            object_key = plugin_object["Key"]
            local_path = os.path.join(
                os.path.abspath("."),
                f"model_plugins/{base64.decodebytes(object_key.encode('utf8')).decode('utf8')}",
            ).replace("-", "_")

            if not overwrite:
                if os.path.isfile(local_path):
                    continue

            response = (
                self.s3_client.get_object(Bucket=self.plugin_bucket, Key=object_key)[
                    "Body"
                ]
                .read()
                .decode("utf8")
            )

            directory = Path(os.path.dirname(local_path))
            directory.mkdir(parents=True, exist_ok=True)

            with open(local_path, "w") as f:
                f.write(response)
