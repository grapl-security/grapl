"""Setuptools script for grapl-model-plugin-deployer"""

import os

from setuptools import setup, find_packages  # type: ignore

HERE = os.path.abspath(os.path.dirname(__file__))


def is_comment(line: str) -> bool:
    """check whether a line is a comment"""
    return line.strip().startswith("#")


with open(os.path.join(HERE, "requirements.txt")) as requirements:
    REQUIREMENTS = tuple(line.strip() for line in requirements if not is_comment(line))

__version__ = "1.0.0"

setup(
    name="grapl-model-plugin-deployer",
    version=__version__,
    author="Grapl, Inc.",
    author_email="grapl.code@graplsecurity.com",
    url="https://github.com/grapl-security/grapl",
    description="Grapl service for deploying plugins",
    packages=find_packages(),
    install_requires=REQUIREMENTS,
    setup_requires=("wheel",),
    zip_safe=False,
)
