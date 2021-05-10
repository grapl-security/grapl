import re
from typing import Final

from pydantic import BaseModel, validator

from grapl_template_generator.common_types import VersionConstraint

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
    pants_black_version_constraint: VersionConstraint
    pants_isort_version_constraint: VersionConstraint
    pants_mypy_version_constraint: VersionConstraint
    lambda_handler: str

    @validator('project_name')
    def service_name_is_kebab(cls, project_name: str) -> str:
        _assert_match('project_name', _PACKAGE_NAME, project_name)
        return project_name

    @validator('pants_version')
    def pants_version_is_valid(cls, pants_version: str) -> str:
        _assert_match('pants_version', _VERSION_RE, pants_version)
        return pants_version

    @validator('pants_python_interpreter_constraints')
    def pants_python_interpreter_constraints_is_valid(cls, pants_python_interpreter_constraints: str) -> str:
        _assert_match('pants_python_interpreter_constraints', _VERSION_CONSTRAINT, pants_python_interpreter_constraints)
        return pants_python_interpreter_constraints

    @validator('pants_black_version_constraint')
    def pants_black_version_constraint_is_valid(cls, pants_black_version_constraint: str) -> str:
        _assert_match('pants_black_version_constraint', _VERSION_CONSTRAINT, pants_black_version_constraint)
        return pants_black_version_constraint

    @validator('pants_isort_version_constraint')
    def pants_isort_version_constraint_is_valid(cls, pants_isort_version_constraint: str) -> str:
        _assert_match('pants_isort_version_constraint', _VERSION_CONSTRAINT, pants_isort_version_constraint)
        return pants_isort_version_constraint

    @validator('pants_mypy_version_constraint')
    def pants_mypy_version_constraint_is_valid(cls, pants_mypy_version_constraint: str) -> VersionConstraint:
        _assert_match('pants_mypy_version_constraint', _VERSION_CONSTRAINT, pants_mypy_version_constraint)
        return VersionConstraint(pants_mypy_version_constraint)
