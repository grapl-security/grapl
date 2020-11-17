from datetime import datetime, timedelta
from typing import NewType

# we have a bunch of bare ints around the codebase
# representing timestamps, this makes it slightly easier
# to reason about
Millis = NewType("Millis", int)

# and for millisecond-durations that have nothing to do with Unix Time:
MillisDuration = NewType("MillisDuration", int)


def as_datetime(millis: Millis) -> datetime:
    return datetime.fromtimestamp(millis / 1000.0)


def as_millis(dt: datetime) -> Millis:
    return Millis(int(dt.timestamp() * 1000))


def as_millis_duration(delta: timedelta) -> MillisDuration:
    return MillisDuration(int(delta.microseconds / 1000))
