FROM postgres:13.4-bullseye

RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=postgreg-apt \
    apt-get update && \
    apt-get -y install --no-install-recommends \
    postgresql-13-partman=4.5.1-1 \
    postgresql-13-cron=1.3.0-2 && \
    rm -rf /var/lib/apt/lists/*

COPY postgres.conf /tmp/postgres.conf
RUN cat /tmp/postgres.conf >> /usr/share/postgresql/13/postgresql.conf.sample