import re
from typing import Final

from pydantic import BaseModel, validator

from grapl_templates.common_types import VersionConstraint

_KEBAB_RE: Final[re.Pattern[str]] = re.compile("[a-z][a-z-]+[a-z]")
_REQUIREMENTS_RE: Final[re.Pattern[str]] = re.compile("[\w][\w\d_]+")


def _assert_match(n: str, regex: re.Pattern[str], to_validate: str) -> None:
    match = regex.match(to_validate)
    if not match:
        raise ValueError(f"Field {n} must match the regex `{regex.pattern}`")

    if match.group(0) != to_validate:
        raise ValueError(f"Field {n} must is invalid, only matched {match}")


class CreatePythonHttpServiceArgs(BaseModel):
    """
    example: 'grapl-analyzerlib'
    """
    package_name: str
    pants_version: VersionConstraint
    pants_python_interpreter_constraints: VersionConstraint
    pants_black_version_constraint: VersionConstraint
    pants_isort_version_constraint: VersionConstraint
    pants_mypy_version_constraint: VersionConstraint
    lambda_handler: str

    @validator('service_name')
    def service_name_is_kebab(cls, service_name: str) -> str:
        _assert_match('service_name', _KEBAB_RE, service_name)
        return service_name

    @validator('pants_version')
    def pants_version_is_valid(cls, pants_version: str) -> str:
        _assert_match('pants_version', _REQUIREMENTS_RE, pants_version)
        return pants_version

    @validator('pants_python_interpreter_constraints')
    def pants_python_interpreter_constraints_is_valid(cls, pants_python_interpreter_constraints: str) -> str:
        _assert_match('pants_python_interpreter_constraints', _REQUIREMENTS_RE, pants_python_interpreter_constraints)
        return pants_python_interpreter_constraints

    @validator('pants_black_version_constraint')
    def pants_black_version_constraint_is_valid(cls, pants_black_version_constraint: str) -> str:
        _assert_match('pants_black_version_constraint', _REQUIREMENTS_RE, pants_black_version_constraint)
        return pants_black_version_constraint

    @validator('pants_isort_version_constraint')
    def pants_isort_version_constraint_is_valid(cls, pants_isort_version_constraint: str) -> str:
        _assert_match('pants_isort_version_constraint', _REQUIREMENTS_RE, pants_isort_version_constraint)
        return pants_isort_version_constraint

    @validator('pants_mypy_version_constraint')
    def pants_mypy_version_constraint_is_valid(cls, pants_mypy_version_constraint: str) -> VersionConstraint:
        _assert_match('pants_mypy_version_constraint', _REQUIREMENTS_RE, pants_mypy_version_constraint)
        return VersionConstraint(pants_mypy_version_constraint)
