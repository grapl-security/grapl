import unittest
from datetime import datetime
from typing_extensions import Final

from grapl_common.time_utils import as_millis, as_datetime

SOME_DT: Final[datetime] = datetime(
    year=2020,
    month=1,
    day=2,
    hour=3,
    minute=4,
    second=5,
    microsecond=6000,  # 6 millis
)


class TimeUtilsTests(unittest.TestCase):
    def test__back_and_forth(self):
        millis = as_millis(SOME_DT)
        assert millis == 1577934245006
        back_to_dt = as_datetime(millis)
        assert SOME_DT == back_to_dt
