"""Setuptools script for Grapl engagement_edge"""

import os

from setuptools import find_packages, setup  # type: ignore

HERE = os.path.abspath(os.path.dirname(__file__))


def is_comment(line):
    """check whether a line is a comment"""
    return line.strip().startswith("#")


with open(os.path.join(HERE, "requirements.txt")) as requirements:
    REQUIREMENTS = tuple(line.strip() for line in requirements if not is_comment(line))

__version__ = "1.0.0"

setup(
    name="engagement_edge",
    version=__version__,
    author="Grapl, Inc.",
    author_email="grapl.code@graplsecurity.com",
    url="https://github.com/grapl-security/grapl",
    description="Grapl edge service for managing engagements",
    packages=find_packages(),
    install_requires=REQUIREMENTS,
    extras_require={
        "typecheck": [
            "mypy",
        ]
    },
    setup_requires=("wheel",),
    zip_safe=False,
)
