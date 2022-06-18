from __future__ import annotations
import base64
import logging
import sys
import os.path

from typing import Optional, TYPE_CHECKING
from pathlib import Path

if TYPE_CHECKING:
    from mypy_boto3_s3 import S3Client

GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


def load_plugins(model_plugins_bucket: str, s3: S3Client, path=None) -> None:
    PluginRetriever(
        plugin_bucket=model_plugins_bucket,
        plugin_directory="./model_plugins/",
        s3_client=s3,
    ).retrieve(overwrite=True, path=path)


class PluginRetriever(object):
    def __init__(
        self,
        plugin_bucket: str,
        plugin_directory: str,
        s3_client: S3Client,
    ) -> None:
        self.plugin_bucket = plugin_bucket
        self.s3_client = s3_client
        self.plugin_directory = plugin_directory

    def retrieve(self, overwrite: bool = False, path: Optional[Path] = None) -> None:
        path = path or "."
        LOGGER.info(f'Writing out plugins to: {os.path.join(path, "model_plugins")}')

        # list plugin files
        plugin_objects = self.s3_client.list_objects(
            Bucket=self.plugin_bucket,
        ).get("Contents", [])

        # Download each one to the /plugins/ directory
        for plugin_object in plugin_objects:
            object_key = plugin_object["Key"]
            plugin_name = object_key.split("/")[0]
            local_path = os.path.join(
                path,
                f"model_plugins/{plugin_name}/{base64.decodebytes(object_key.split('/')[1].encode('utf8')).decode('utf8')}",
            ).replace("-", "_")

            if local_path[-1] == "/":
                local_path = local_path[:-1]

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
