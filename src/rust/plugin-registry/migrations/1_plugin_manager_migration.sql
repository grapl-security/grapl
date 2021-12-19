CREATE TABLE IF NOT EXISTS plugins
(
    plugin_id        uuid         PRIMARY KEY,
    display_name     varchar(128) NOT NULL,
    tenant_id        uuid         NOT NULL,
    plugin_type      varchar(255) NOT NULL,
    artifact_s3_key  varchar(1024) NOT NULL
);