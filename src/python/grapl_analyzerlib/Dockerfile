FROM grapl/grapl-python-build:latest AS grapl-analyzerlib-python-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . grapl_analyzerlib
COPY --from=grapl/grapl-graph-descriptions-python-build /home/grapl/venv venv

# Install requirement `grapl_common` - we could also manually COPY the `venv/site_packages`, but the pip install
# is cleaner.
COPY --from=grapl/grapl-common-python-build /home/grapl/grapl_common grapl_common
RUN /bin/bash -c "source venv/bin/activate && cd grapl_common && pip install ."

RUN /bin/bash -c "source venv/bin/activate && cd grapl_analyzerlib && pip install ."
RUN /bin/bash -c "source venv/bin/activate && cd grapl_analyzerlib && python setup.py sdist bdist_wheel"
