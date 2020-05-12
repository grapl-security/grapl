from typing import NewType
from datetime import datetime

# we have a bunch of bare ints around the codebase
# representing timestamps, this makes it slightly easier
# to reason about
Millis = NewType("Millis", int)


def as_datetime(millis: Millis) -> datetime:
    return datetime.fromtimestamp(millis / 1000.0)


def as_millis(dt: datetime) -> Millis:
    return Millis(dt.timestamp() * 1000)
