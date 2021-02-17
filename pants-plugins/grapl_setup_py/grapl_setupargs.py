import logging

from pants.backend.python.goals.setup_py import SetupKwargs, SetupKwargsRequest
from pants.engine.fs import DigestContents, GlobMatchErrorBehavior, PathGlobs
from pants.engine.rules import Get, collect_rules, rule
from pants.engine.target import Target
from pants.engine.unions import UnionRule

logger = logging.getLogger(__name__)

# These setup.py arguments will be used by ALL Python distributions
# created in this repository.
HARDCODED_KWARGS = {
    "author": "Grapl, Inc.",
    "author_email": "grapl-code@graplsecurity.com",
    "maintainer": "Grapl, Inc.",
    "maintainer_email": "grapl-code@graplsecurity.com",
    "url": "https://github.com/grapl-security/grapl",
    "project_urls": {
        "Documentation": "https://grapl.readthedocs.io",
        "Source": "https://github.com/grapl-security/grapl",
        "Tracker": "https://github.com/grapl-security/grapl/issues",
    },
    "license": "MIT",
    "classifiers": [
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
    ],
}


class GraplSetupKwargsRequest(SetupKwargsRequest):
    @classmethod
    def is_applicable(cls, _: Target) -> bool:
        """Use this for ALL Python distributions created in this repository."""
        return True


@rule
async def setup_kwargs_plugin(request: GraplSetupKwargsRequest) -> SetupKwargs:

    explicit_kwargs = request.explicit_kwargs

    if "name" not in explicit_kwargs:
        raise ValueError(
            f"Must provide a `name` key in the `provides` field for {request.target.address}"
        )

    if "description" not in explicit_kwargs:
        raise ValueError(
            f"Must provide a `description` key in the `provides` field for {request.target.address}"
        )

    if "version" not in explicit_kwargs:
        raise ValueError(
            f"Must provide a `version` key in the `provides` field for {request.target.address}"
        )

    # Look for a README.md file as a sibling to the BUILD file this
    # target is defined in.
    default_readme_path = f"{request.target.address.spec_path}/README.md"

    if "long_description" in explicit_kwargs:
        raise ValueError(
            f"Do not provide a `long_description` in the `provides` field for {request.target.address}. "
            f"Instead, either place a `README.md` file at {default_readme_path} "
            "OR specify a path to an appropriate Markdown file, relative to the Pants root "
            "in the `readme` key in the `provides` field"
        )

    # "readme" is a key that we (Grapl) use; it's not in standard
    # Pants. There may be some changes coming soon, though:
    # https://github.com/pantsbuild/pants/issues/11554
    readme_path = (
        explicit_kwargs.pop("readme")
        if "readme" in explicit_kwargs
        else default_readme_path
    )

    logger.info(f"Reading long_description from {readme_path}")
    digest_contents = await Get(
        DigestContents,
        PathGlobs(
            [readme_path],
            description_of_origin=f"README resolution in `setup_py()` plugin ({__file__}) for {request.target.address}",
            glob_match_error_behavior=GlobMatchErrorBehavior.error,
        ),
    )
    long_description = "\n".join(
        file_content.content.decode() for file_content in digest_contents
    )
    explicit_kwargs["long_description"] = long_description

    # Set hardcoded values, raising an exception if any of them are
    # overridden by the user.
    conflicts = set(explicit_kwargs.keys()).intersection(HARDCODED_KWARGS.keys())
    if conflicts:
        raise ValueError(
            f"These kwargs should not be set in the `provides` field for {request.target.address} "
            "because our internal plugin will automatically set them: "
            f"{sorted(conflicts)}"
        )
    explicit_kwargs.update(HARDCODED_KWARGS)

    return SetupKwargs(explicit_kwargs, address=request.target.address)


def rules():
    return [
        *collect_rules(),
        UnionRule(SetupKwargsRequest, GraplSetupKwargsRequest),
    ]
