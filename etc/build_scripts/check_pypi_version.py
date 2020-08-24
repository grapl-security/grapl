# This script is meant to be used in the build system to determine
# whether a python package needs to have its version bumped before
# being released. You can also use it locally if you wish. It will
# exit(0) if the version does not need to be bumped, otherwise
# exit(1) and print a message to STDERR.
#
# usage:
#   python check_pypi_version.py <package> <current_version> [<test_pypi>]
#
# examples:
#   python check_pypi_version.py grapl_graph_descriptions 1.0.0 False
#   python check_pypi_version.py grapl_analyzerlib 0.2.63

import sys

import pypi_simple


def needs_version_bumped(package: str, current_version: str, test_pypi: bool) -> bool:
    client = (
        pypi_simple.PyPISimple("https://test.pypi.org/simple/")
        if test_pypi
        else pypi_simple.PyPISimple()
    )

    project_files = client.get_project_files(package)
    project_files = sorted(
        (f for f in project_files if f.yanked is None),
        key=lambda p: tuple(map(int, p.version.split("."))),
        reverse=True,
    )
    if not project_files:
        raise Exception(
            f"Couldn't find project files for package={package}, test_pypi={test_pypi}"
        )
    latest_version = project_files[0].version.strip()

    current_version = current_version.strip()
    current_version_parts = current_version.split(".")
    latest_version_parts = latest_version.split(".")
    if latest_version == current_version:
        return (True, latest_version)
    elif int(latest_version_parts[0]) > int(current_version_parts[0]):
        return (True, latest_version)
    elif int(latest_version_parts[1]) > int(current_version_parts[1]):
        return (True, latest_version)
    elif int(latest_version_parts[2]) >= int(current_version_parts[2]):
        return (True, latest_version)
    else:
        return (False, latest_version)


def main(package, current_version, test_pypi):
    must_bump, latest_version = needs_version_bumped(
        package=package, current_version=current_version, test_pypi=test_pypi
    )

    if must_bump:
        test_str = "Test" if test_pypi else ""
        sys.stderr.write(
            f"{package} {current_version} needs version bump."
            f" Latest version in {test_str} PyPI: {latest_version}\n"
        )
        sys.exit(1)
    else:
        sys.exit(0)


if __name__ == "__main__":
    PACKAGE = sys.argv[1]
    CURRENT_VERSION = sys.argv[2]
    TEST_PYPI = bool(sys.argv[3]) if len(sys.argv) > 3 else False

    main(package=PACKAGE, current_version=CURRENT_VERSION, test_pypi=TEST_PYPI)
