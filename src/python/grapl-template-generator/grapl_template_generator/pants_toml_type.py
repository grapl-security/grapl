from typing import List

from typing_extensions import TypedDict

PantsGlobal = TypedDict(
    "PantsGlobal",
    {
        "pants_version": str,
        "backend_packages": List[str],
        "pythonpath": List[str],
        "build_file_prelude_globs": List[str],
        "pants_ignore": List[str],
    },
)

PantsSource = TypedDict("PantsSource", {"root_patterns": List[str]})

PantsToml = TypedDict(
    "PantsToml",
    {
        "GLOBAL": PantsGlobal,
        "source": PantsSource,
    },
)
