from typing import Optional, Union


def to_bool(input: Optional[Union[str, bool]]) -> Optional[bool]:
    if isinstance(input, bool):
        return input
    elif input is None:
        return None
    elif input in ("True", "true"):
        return True
    elif input in ("False", "false"):
        return False
    else:
        raise ValueError(f"Invalid bool value: {input}")
