"""Setuptools script for Grapl engagement_creator"""

import os

from setuptools import setup

HERE = os.path.abspath(os.path.dirname(__file__))


def is_comment(line):
    """check whether a line is a comment"""
    return line.strip().startswith("#")


with open(os.path.join(HERE, "requirements.txt")) as requirements:
    REQUIREMENTS = tuple(line.strip() for line in requirements if not is_comment(line))

with open(os.path.join(HERE, "requirements-test.txt")) as requirements_test:
    REQUIREMENTS_TEST = tuple(
        line.strip() for line in requirements_test if not is_comment(line)
    )

__version__ = "1.0.0"

setup(
    name="engagement_creator",
    version=__version__,
    author="Grapl, Inc.",
    author_email="grapl.code@graplsecurity.com",
    url="https://github.com/grapl-security/grapl",
    description="Grapl service which creates engagements",
    install_requires=REQUIREMENTS,
    tests_require=REQUIREMENTS_TEST,
    setup_requires=("wheel",),
    zip_safe=False,
)
