import os.path

from mypy_boto3 import s3


class PluginRetriever(object):
    def __init__(
            self,
            plugin_bucket: str,
            plugin_directory: str,
            s3_client: s3.S3Client,
    ) -> None:
        self.plugin_bucket = plugin_bucket
        self.s3_client = s3_client
        self.plugin_directory = plugin_directory

    def retrieve(self, overwrite: bool = False) -> None:
        # list plugin files
        plugin_objects = self.s3_client.list_objects(
            Bucket=self.plugin_bucket,
        )

        # Download each one to the /plugins/ directory
        for plugin_object in plugin_objects:
            local_path = f"./plugins{plugin_object}"

            if not overwrite:
                if os.path.isfile(local_path):
                    print(f"./plugins{plugin_object} already exists")
                    continue

            response = (
                self.s3_client.get_object(
                    Bucket=self.plugin_bucket,
                    Key=plugin_object
                )
                ['Body'].read()
            )

            print(f"Writing plugin to: ./plugins{plugin_object}")
            with open(local_path, 'w') as f:
                f.write(response)
