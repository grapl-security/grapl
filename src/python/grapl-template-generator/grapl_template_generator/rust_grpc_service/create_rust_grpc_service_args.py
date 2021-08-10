from __future__ import annotations

import re

from pydantic import BaseModel, validator
from typing_extensions import Final

_CRATE_NAME: Final[re.Pattern[str]] = re.compile("[-a-z]*")


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

    crate_name: str

    @validator("crate_name")
    def validate_package_name(cls, crate_name: str) -> str:
        _assert_match("crate_name", _CRATE_NAME, crate_name)
        return crate_name
