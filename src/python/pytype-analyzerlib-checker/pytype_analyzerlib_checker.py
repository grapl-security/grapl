import subprocess
import sys


def _typecheck_analyzerlib(pytype_config_rel_path: str, analyzerlib_rel_path: str) -> int:
    return subprocess.check_call(
        ["pytype", "--config", f"{pytype_config_rel_path}", f"{analyzerlib_rel_path}"]
    )


def main() -> None:
    sys.exit(_typecheck_analyzerlib(
        "src/python/grapl_analyzerlib/pytype.cfg",
        "src/python/grapl_analyzerlib/"
    ))


if __name__ == "__main__":
    main()
