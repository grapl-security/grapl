FROM postgres:13.4-bullseye

RUN --mount=type=cache,target=/var/lib/apt/lists \
    apt-get update && \
    apt-get -y install --no-install-recommends \
    postgresql-13-partman=4.5.1-1 \
    postgresql-13-cron=1.3.0-2 && \
    rm -rf /var/lib/apt/lists/*

RUN echo "shared_preload_libraries = 'pg_cron'" >> /usr/share/postgresql/13/postgresql.conf.sample && \
    echo "cron.database_name = 'postgres'" >> /usr/share/postgresql/13/postgresql.conf.sample
