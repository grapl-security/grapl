import unittest
from datetime import datetime

from grapl_common.time_utils import as_datetime, as_millis
from typing_extensions import Final

SOME_DT: Final[datetime] = datetime(
    year=2020, month=1, day=2, hour=3, minute=4, second=5, microsecond=6000,  # 6 millis
)


class TestTimeUtils(unittest.TestCase):
    def test__back_and_forth(self) -> None:
        millis = as_millis(SOME_DT)
        back_to_dt = as_datetime(millis)
        assert SOME_DT == back_to_dt
