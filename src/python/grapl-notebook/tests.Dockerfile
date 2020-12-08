# This file will improve once #444 lands
# primarily to use pre-downloaded tools instead of downloading again

FROM grapl/grapl-notebook
WORKDIR /home/grapl
RUN source venv/bin/activate && pip install nbqa mypy
RUN source venv/bin/activate && cd grapl-notebook && nbqa mypy Demo_Engagement.ipynb