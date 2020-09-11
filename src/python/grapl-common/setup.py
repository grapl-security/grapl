# type: ignore
import os

from setuptools import setup, find_packages

with open("README.md") as readme:
    README = readme.read()

HERE = os.path.abspath(os.path.dirname(__file__))


def find_version():
    with open(os.path.join(HERE, "VERSION")) as version:
        return version.read().strip()


__version__ = find_version()

setup(
    name="grapl-common",
    version=__version__,
    description="Shared code between Grapl components",
    long_description=README,
    url="https://github.com/grapl-security/grapl",
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
        "grapl-common": [
            "py.typed",
        ]
    },
    include_package_data=True,
    install_requires=[
        # We might want this in the future
        # "grapl_graph_descriptions==1.0.*",
        "typing_extensions",
    ],
    # We'll probably have some dataclasses in here in the future
    python_requires=">=3.6",
    extras_require={
        "typecheck": [
            "mypy",
        ],
    },
)
