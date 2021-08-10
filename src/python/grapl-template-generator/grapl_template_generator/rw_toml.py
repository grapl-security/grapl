from pathlib import Path
from typing import Any, Generic, Mapping, TypeVar, cast

import toml as toml_poorly_typed
from typing_extensions import Final

# Sadly, the toml library's type stubs are just incorrect.
# https://github.com/uiri/toml/pull/347
toml = cast(Any, toml_poorly_typed)

T = TypeVar("T", bound=Mapping)


class ReadWriteToml(Generic[T]):
    """
    Wraps any r/w with the toml with the preserve-comment codec.
    It *DOES NOT WORK VERY WELL*.
    https://github.com/uiri/toml/issues/371

    Use:
    t = TomlLoader(some_path)
    t.loaded_toml["stuff"] = "some_mutation"
    t.write()
    """

    def __init__(self, path: Path) -> None:
        loaded_toml = cast(
            T, toml.load(path, decoder=toml.decoder.TomlPreserveCommentDecoder())
        )
        self.path = path
        self.loaded_toml: Final[T] = loaded_toml

    def write(self) -> None:
        with open(self.path, "w") as f:
            toml_str = toml.dumps(
                self.loaded_toml, encoder=toml.encoder.TomlPreserveCommentEncoder()
            )
            f.write(toml_str)
