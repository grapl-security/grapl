CREATE TABLE IF NOT EXISTS plugin_artifacts
(
    artifact_id         uuid          PRIMARY KEY,
    artifact_version    integer       NOT NULL,
    artifact_s3_key     varchar(1024) NOT NULL
);

CREATE TABLE IF NOT EXISTS plugins
(
    plugin_id        uuid PRIMARY KEY,
    display_name     varchar(128) NOT NULL,
    tenant_id        uuid         NOT NULL,
    artifact_id      uuid         NOT NULL,
    plugin_type      varchar(255) NOT NULL,
    UNIQUE(
              display_name,
              tenant_id,
              artifact_id,
              plugin_type
          ),
    CONSTRAINT fk_plugin_artifact
        FOREIGN KEY (artifact_id)
        REFERENCES plugin_artifacts (artifact_id)
    ON DELETE CASCADE
);