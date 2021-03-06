import os

from setuptools import setup, find_packages  # type: ignore

TEST_REQUIREMENTS = [
    "pytest",
    # simultaneously run pytest on multiple cores with `pytest -n NUMCORES`
    "pytest-xdist",
    "pytest-cov",
    "hypothesis",
]

with open("README.md") as readme:
    README = readme.read()

HERE = os.path.abspath(os.path.dirname(__file__))


def find_version():
    with open(os.path.join(HERE, "VERSION")) as version:
        return version.read().strip()


__version__ = find_version()

setup(
    name="grapl_analyzerlib",
    version=__version__,
    description="Library for Grapl Analyzers",
    long_description=README,
    url="https://github.com/insanitybit/grapl_analyzerlib/",
    author="insanitybit",
    author_email="insanitybit@gmail.com",
    license="MIT",
    classifiers=[
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
    ],
    zip_safe=False,
    packages=find_packages(),
    package_data={
        "grapl_analyzerlib": ["py.typed"],
        "grapl_analyzerlib.nodes": ["nodes/py.typed"],
        "grapl_analyzerlib.schemas": ["schemas/py.typed"],
    },
    include_package_data=True,
    install_requires=[
        "boto3",
        "grapl_common",
        "graplinc_grapl_api",
        "pydgraph",
        "typing_extensions",
    ],
    extras_require={
        "test": TEST_REQUIREMENTS,
        "typecheck": TEST_REQUIREMENTS + ["pytype"],
    },
)
