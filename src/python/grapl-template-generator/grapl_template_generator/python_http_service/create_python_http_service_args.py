from __future__ import annotations

import re

from grapl_template_generator.common_types import VersionConstraint
from pydantic import BaseModel, validator
from typing_extensions import Final

_PACKAGE_NAME: Final[re.Pattern[str]] = re.compile("[A-Z][a-z]*(\s[A-Z][a-z]*)*")
_VERSION_CONSTRAINT: Final[re.Pattern[str]] = re.compile("[\w][\w\d_]+==[ab\d\*\.]+")
_VERSION_RE: Final[re.Pattern[str]] = re.compile("\d+\.\d+\.\d+")


def _assert_match(n: str, regex: re.Pattern[str], to_validate: str) -> None:
    match = regex.match(to_validate)
    if not match:
        raise ValueError(f"Field {n} must match the regex `{regex.pattern}`")

    if match.group(0) != to_validate:
        raise ValueError(f"Field {n} must is invalid, only matched {match}")


class CreatePythonHttpServiceArgs(BaseModel):
    """
    example: 'My Python Service'
    """

    project_name: str
    pants_version: str
    pants_python_interpreter_constraints: VersionConstraint
    lambda_handler: str

    @validator("project_name")
    def service_name_is_kebab(cls, project_name: str) -> str:
        _assert_match("project_name", _PACKAGE_NAME, project_name)
        return project_name

    @validator("pants_version")
    def pants_version_is_valid(cls, pants_version: str) -> str:
        _assert_match("pants_version", _VERSION_RE, pants_version)
        return pants_version

    @validator("pants_python_interpreter_constraints")
    def pants_python_interpreter_constraints_is_valid(
        cls, pants_python_interpreter_constraints: str
    ) -> str:
        _assert_match(
            "pants_python_interpreter_constraints",
            _VERSION_CONSTRAINT,
            pants_python_interpreter_constraints,
        )
        return pants_python_interpreter_constraints
