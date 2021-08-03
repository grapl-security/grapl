from __future__ import annotations

import re
from typing import Union

from grapl_template_generator.common_types import VersionConstraint
from pydantic import BaseModel, validator
from typing_extensions import Final, Literal

_CRATE_NAME: Final[re.Pattern[str]] = re.compile("[-a-z]*")
_VERSION_RE: Final[re.Pattern[str]] = re.compile("\d+\.\d+\.\d+")


def _assert_match(n: str, regex: re.Pattern[str], to_validate: str) -> None:
    match = regex.match(to_validate)
    if not match:
        raise ValueError(f"Field {n} must match the regex `{regex.pattern}`")

    if match.group(0) != to_validate:
        raise ValueError(f"Field {n} must is invalid, only matched {match}")


class CreateRustGrpcServiceArgs(BaseModel):
    """
    example: 'graph-mutation-service'
    """

    package_name: str
    cargo_version: VersionConstraint
    rustc_channel: Union[Literal["nightly"], Literal["stable"], Literal["beta"]]

    @validator("package_name")
    def validate_package_name(cls, package_name: str) -> str:
        _assert_match("package_name", _CRATE_NAME, package_name)
        return package_name

    @validator("cargo_version")
    def validate_cargo_version(cls, cargo_version: str) -> str:
        _assert_match("cargo_version", _VERSION_RE, cargo_version)
        return cargo_version

    @validator("rustc_channel")
    def validate_rustc_channel(cls, rustc_channel: str) -> str:
        if rustc_channel not in ("stable", "beta", "nightly"):
            raise ValueError(
                f"Must set valid rustc_channel - stable, beta, or nightly. Not {rustc_channel}"
            )
        return rustc_channel
