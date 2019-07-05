from setuptools import setup

setup(
    name="grapl_analyzerlib",
    version="0.1.67",
    description="Library for Grapl Analyzers",
    url="https://github.com/insanitybit/grapl/",
    author="insanitybit",
    author_email="insanitybit@gmail.com",
    license="MIT",
    classifiers=[
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
    ],
    packages=["grapl_analyzerlib"],
    include_package_data=True,
    install_requires=["pydgraph"],
)
