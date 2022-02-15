from pathlib import Path


def path_from_root(from_root: str) -> Path:
    """
    To access something in the Grapl repo root, we have to reference two dirs
    up because the Pulumi cwd is `${GRAPL_ROOT}/pulumi/grapl`
    """
    return Path("../.." / from_root)
