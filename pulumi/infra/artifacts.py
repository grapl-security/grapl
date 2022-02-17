from __future__ import annotations

from typing import Mapping, Optional

from infra import config

import pulumi


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
                f"Expected to find an artifacts entry for {key} in "
                f"{config.STACK_CONFIG_FILENAME}"
            )
        return None

    @staticmethod
    def from_config(pulumi_config: pulumi.Config) -> ArtifactGetter:
        """
        If local-grapl:
            - We don't require an `artifacts:` field
            - `.get()` can be None
        Else:
            - We require an `artifacts:` field
            - `.get()` must resolve a value for that key or raise KeyError
        """
        artifact_dict = (
            pulumi_config.get_object("artifacts") or {}
            if config.LOCAL_GRAPL
            else pulumi_config.require_object("artifacts")
        )
        return ArtifactGetter(
            artifacts=artifact_dict,
            require_artifact=(not config.LOCAL_GRAPL),
        )
