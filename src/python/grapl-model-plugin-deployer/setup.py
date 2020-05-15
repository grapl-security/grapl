"""Setuptools script for grapl-model-plugin-deployer"""

import os

from setuptools import setup

HERE = os.path.abspath(os.path.dirname(__file__))


def is_comment(line):
    """check whether a line is a comment"""
    return line.strip().startswith('#')


with open(os.path.join(HERE, 'requirements.txt')) as requirements:
    REQUIREMENTS = tuple(
        line.strip() for line in requirements if not is_comment(line)
    )

with open(os.path.join(HERE, 'requirements-test.txt')) as requirements_test:
    REQUIREMENTS_TEST = tuple(
        line.strip() for line in requirements_test if not is_comment(line)
    )

__version__ = '1.0.0'

setup(
    name='grapl-model-plugin-deployer',
    version=__version__,
    author='Grapl, Inc.',
    author_email='FIXME',
    url='https://github.com/grapl-security/grapl',
    description='Grapl service for deploying plugins',
    install_requires=REQUIREMENTS,
    tests_require=REQUIREMENTS_TEST,
    zip_safe=False
)
