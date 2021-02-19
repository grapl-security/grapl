# This file was inspired by
# https://github.com/python-poetry/poetry/discussions/1879#discussioncomment-216865
# FIXME: move this image's configuration into grapl-python-build
FROM python:3.7-slim-buster as graplctl-python-base
ENV PYTHONUNBUFFERED=1 \
        PYTHONDONTWRITEBYTECODE=1 \
        PIP_NO_CACHE_DIR=off \
        PIP_DISABLE_PIP_VERSION_CHECK=on \
        PIP_DEFAULT_TIMEOUT=100 \
        # https://python-poetry.org/docs/configuration/#using-environment-variables
        POETRY_VERSION=1.0.3 \
        POETRY_HOME="/opt/poetry" \
        # create poetry virtual environment `.venv` in project root
        POETRY_VIRTUALENVS_IN_PROJECT=true \
        POETRY_NO_INTERACTION=1 \
        # paths for requirements + virtual environment
        PYSETUP_PATH="/opt/pysetup" \
        VENV_PATH="/opt/pysetup/.venv"
# prepend poetry and venv to path
ENV PATH="$POETRY_HOME/bin:$VENV_PATH/bin:$PATH"

# `graplctl-builder-base` stage is used to build deps + create our virtual environment
FROM graplctl-python-base as graplctl-builder-base
RUN apt-get update \
    && apt-get install --no-install-recommends -y \
        # deps for installing poetry
        curl \
        # deps for building python deps
        build-essential
# install poetry - respects $POETRY_VERSION & $POETRY_HOME
RUN curl -sSL https://raw.githubusercontent.com/sdispater/poetry/master/get-poetry.py | python
# copy project requirement files here to ensure they will be cached.
WORKDIR $PYSETUP_PATH
COPY poetry.lock pyproject.toml ./
# install runtime deps - uses $POETRY_VIRTUALENVS_IN_PROJECT internally
RUN poetry install --no-dev

# `graplctl-build` image is used during build/testing
FROM graplctl-python-base as graplctl-build
ENV FASTAPI_ENV=development
WORKDIR $PYSETUP_PATH
# copy in our built poetry + venv
COPY --from=graplctl-builder-base $POETRY_HOME $POETRY_HOME
COPY --from=graplctl-builder-base $PYSETUP_PATH $PYSETUP_PATH
# quicker install as runtime deps are already installed
WORKDIR /graplctl
COPY . .
RUN poetry install
# no-op command to make sure we don't exec anything accidentally
CMD :
