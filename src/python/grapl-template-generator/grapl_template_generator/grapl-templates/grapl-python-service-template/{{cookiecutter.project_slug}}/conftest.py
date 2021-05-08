import os
import tempfile

import hypothesis
import pytest

hypothesis.configuration.set_hypothesis_home_dir(
    os.path.join(tempfile.gettempdir(), ".hypothesis")
)


hypothesis.settings.register_profile("ci", max_examples=1000)
hypothesis.settings.register_profile("dev", max_examples=10)
hypothesis.settings.register_profile("debug", max_examples=10, verbosity=hypothesis.Verbosity.verbose, deadline=None)
hypothesis.settings.load_profile(os.getenv(u"HYPOTHESIS_PROFILE", "default"))
