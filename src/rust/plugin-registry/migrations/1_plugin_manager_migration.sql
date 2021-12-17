CREATE TABLE IF NOT EXISTS plugin_artifacts
(
    artifact_id         uuid          PRIMARY KEY,
    artifact_version    integer       NOT NULL,
    artifact_s3_key     varchar(1024) NOT NULL
);

CREATE TABLE IF NOT EXISTS plugins
(
    plugin_key       serial PRIMARY KEY,
    plugin_id        uuid UNIQUE  NOT NULL,
    tenant_id        uuid         NOT NULL,
    configuration_id uuid         NOT NULL,
    artifact_id      uuid         NOT NULL,
    plugin_type      varchar(255) NOT NULL,
    CONSTRAINT fk_plugin_artifact
        FOREIGN KEY (artifact_id)
            REFERENCES plugin_artifacts (artifact_id)
            ON DELETE CASCADE
);