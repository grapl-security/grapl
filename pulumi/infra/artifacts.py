from typing import Mapping, Optional


class ArtifactGetter:
    """
    A facade in front of the `artifacts` in Pulumi.stackname.yaml that
    abstracts out the "sometimes we don't require that a key exists" aspect
    that is prevalent in Local Grapl development.
    """

    def __init__(
        self,
        artifacts: Mapping[str, str],
        require_artifact: bool = False,
    ) -> None:
        self.artifacts = artifacts
        self.require_artifact = require_artifact

    def get(self, key: str) -> Optional[str]:
        """
        Try and get the value from artifacts.
        If no such artifact exists, and require_artifact, throw error

        We generally set `require_artifact=True` for production deployments.
        """
        artifact_version = self.artifacts.get(key)
        if artifact_version is not None:
            return artifact_version
        if self.require_artifact:
            raise KeyError(
                f"Expected to find an artifacts entry for {key} in Pulumi config file"
            )
        return None
