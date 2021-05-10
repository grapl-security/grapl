import re
from typing import Final, Literal, Union

from pydantic import BaseModel, validator

from grapl_template_generator.common_types import VersionConstraint

# grapl-templates/rust-grpc-service/cookiecutter.json
# {
#     "project_name": "My New Project",
#     "project_slug": "{{ cookiecutter.project_name|lower|replace(' ', '-') }}",
#     "service_name": "{{ cookiecutter.project_name|replace(' ', '') }}",
#     "snake_project_name": "{{ cookiecutter.project_slug|replace('-', '_') }}",
#     "cargo-version": "1.52.0",
#     "rustc_channel": "stable",
#     "proto_path" : "proto/"
# }


_CRATE_NAME: Final[re.Pattern[str]] = re.compile("[-a-z]*")
_VERSION_CONSTRAINT: Final[re.Pattern[str]] = re.compile("[\w][\w\d_]+=[ab\d\*\.]+")
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

    @validator('package_name')
    def validate_package_name(cls, package_name: str) -> str:
        _assert_match('package_name', _CRATE_NAME, package_name)
        return package_name

    @validator('cargo_version')
    def validate_cargo_version(cls, cargo_version: str) -> str:
        _assert_match('cargo_version', _VERSION_RE, cargo_version)
        return cargo_version

    @validator('rustc_channel')
    def validate_rustc_channel(cls, rustc_channel: str) -> str:
        if rustc_channel not in ('stable', 'beta', 'nightly'):
            raise ValueError(f"Must set valid rustc_channel - stable, beta, or nightly. Not {rustc_channel}")
        return rustc_channel
