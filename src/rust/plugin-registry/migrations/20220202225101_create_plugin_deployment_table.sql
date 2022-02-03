-- Satisfies the "If success, mark plugin as being deployed in plugins table"
-- requirement of RFC 0000 DeployPlugin.

DO $$ BEGIN
    IF to_regtype('plugin_deployment_status') IS NULL THEN
        CREATE TYPE plugin_deployment_status AS ENUM ('fail', 'success');
    END IF;
END $$;


CREATE TABLE IF NOT EXISTS plugin_deployment
(
    id               SERIAL                    PRIMARY KEY,
    plugin_id        uuid                      NOT NULL,
    deploy_time      timestamptz               NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status           plugin_deployment_status  NOT NULL
);

CREATE INDEX IF NOT EXISTS plugin_key_ix ON plugin_deployment (plugin_id);