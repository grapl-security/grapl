FROM postgres:13.4-bullseye

RUN apt update && \
    apt -y install \
    postgresql-13-partman \
    postgresql-13-cron


RUN echo "shared_preload_libraries = 'pg_cron'" >> /usr/share/postgresql/13/postgresql.conf.sample
RUN echo "cron.database_name = 'postgres'" >> /usr/share/postgresql/13/postgresql.conf.sample
