# This file will improve once #444 lands
# primarily to use pre-downloaded tools instead of downloading again

FROM grapl/grapl-notebook
WORKDIR /home/grapl
RUN source venv/bin/activate && pip install nbqa mypy boto3-stubs[essential]
RUN source venv/bin/activate && cd grapl-notebook && nbqa mypy Demo_Engagement.ipynb

# While the grapl provision notebook technically has nothing to do with grapl-notebook,
# I've arbitrarily decided to make this the generalized "check the quality of ipynbs" image.

COPY --from=grapl/etc-build /home/grapl/etc /home/grapl/etc
RUN source venv/bin/activate && cd etc && nbqa mypy Grapl\ Provision.ipynb