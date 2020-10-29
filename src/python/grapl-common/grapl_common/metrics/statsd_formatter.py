import re
from typing import Pattern, Sequence, Union

from typing_extensions import Final, Literal

DEFAULT_SAMPLE_RATE: Final[float] = 1.0

_INVALID_CHARS: Final[Pattern] = re.compile(r"[|#,=:]")

MetricType = Literal["g", "c", "ms", "h"]  # |m is also valid, but I chose to ignore it


class TagPair:
    tag_key: str
    tag_value: str

    def __init__(self, tag_key: str, tag_value: str) -> None:
        _reject_invalid_chars(tag_key)
        _reject_invalid_chars(tag_value)
        self.tag_key = tag_key
        self.tag_value = tag_value

    def statsd_serialized(self) -> str:
        return ":".join((self.tag_key, self.tag_value))


def _reject_invalid_chars(s: str) -> None:
    # TODO - consider a cache of acceptable strings, since python inters its strings
    match = _INVALID_CHARS.search(s)
    if match:
        raise ValueError(f"Invalid character in input {s}")


def statsd_format(
    metric_name: str,
    value: Union[int, float],
    typ: MetricType,
    sample_rate: float = DEFAULT_SAMPLE_RATE,
    tags: Sequence[TagPair] = (),
) -> str:
    """
    Mainline `statsd` hasn't chosen a tag syntax yet: https://github.com/statsd/statsd/issues/619
    However, it looks like they will be supporting the Graphite and DogStatsD formats.
    I've arbitrarily chosen the DogStatsD format.

    You can find the spec here:
    https://github.com/b/statsd_spec

    <METRIC_NAME>:<VALUE>|<TYPE>|@<SAMPLE_RATE>|#<TAG_KEY_1>:<TAG_VALUE_1>,<TAG_2>
    """
    _reject_invalid_chars(metric_name)

    # sections will eventually be joined by |
    sections = [f"{metric_name}:{value}", typ]

    # Add sample rate.
    # Counter - 'c' - is the only metric that responds to sample rate
    if typ == "c" and sample_rate != DEFAULT_SAMPLE_RATE:
        if not (0.0 <= sample_rate <= 1.0):
            raise ValueError(f"Bad sample rate {sample_rate}")
        sections.append(f"@{sample_rate}")

    # Add tags
    tag_section = ",".join(tag_pair.statsd_serialized() for tag_pair in tags)
    if tag_section:
        sections.append(f"#{tag_section}")
    return "|".join(sections)
