CREATE SCHEMA IF NOT EXISTS plugin_work_queue;

DO $$ BEGIN
    IF to_regtype('plugin_work_queue.status') IS NULL THEN
        CREATE TYPE plugin_work_queue.status AS ENUM ('enqueued', 'failed', 'processed');
    END IF;
END $$;

CREATE OR REPLACE FUNCTION plugin_work_queue.megabytes(bytes integer) RETURNS integer AS
$$
BEGIN
    RETURN bytes * 1000 * 1000;
END;
$$ LANGUAGE plpgsql
    IMMUTABLE;

CREATE TABLE IF NOT EXISTS plugin_work_queue.generator_plugin_executions
(
    --`execution_key` is a `bigserial` - an auto-incrementing 64bit integer.
    execution_key    bigserial                NOT NULL,
    -- `tenant_id` is a unique identifier for the tenant for which this job belongs
    tenant_id        uuid                     NOT NULL,
    -- `plugin_id` is the uuid name of the plugin to send this execution result to.
    plugin_id        uuid                     NOT NULL,
    -- `pipeline_message` is the raw bytes to be interpreted by the plugin.
    pipeline_message bytea                    NOT NULL,
    -- `current_status` is an enum representing the state. 'enqueued' is the default, and a message will be set to either 'processed' or 'failed' based on its success.
    current_status   plugin_work_queue.status NOT NULL,
    -- `creation_time` is when the row was created.
    creation_time    timestamptz                NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- `last_updated` is set with each update to the row
    last_updated     timestamptz                NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- `visible_after` is essentially a visibility timeout. See the `Visibility Timeout` section below.
    visible_after    timestamptz                NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- `try_count` on every receive we increment `try_count` to indicate another attempt to process this message
    try_count        integer                  NOT NULL,
    -- We limit the message to 256MB, which is arbitrary but reasonable as an upper limit
    CHECK (length(pipeline_message) < plugin_work_queue.megabytes(256)),
    CHECK (last_updated >= creation_time)
)
;

CREATE TABLE IF NOT EXISTS plugin_work_queue.analyzer_plugin_executions
(
    --`execution_key` is a `bigserial` - an auto-incrementing 64bit integer.
    execution_key    bigserial                NOT NULL,
    -- `tenant_id` is a unique identifier for the tenant for which this job belongs
    tenant_id        uuid                     NOT NULL,
    -- `plugin_id` is the uuid name of the plugin to send this execution result to.
    plugin_id        uuid                     NOT NULL,
    -- `pipeline_message` is the raw bytes to be interpreted by the plugin.
    pipeline_message bytea                    NOT NULL,
    -- `current_status` is an enum representing the state. 'enqueued' is the default, and a message will be set to either 'processed' or 'failed' based on its success.
    current_status   plugin_work_queue.status NOT NULL,
    -- `creation_time` is when the row was created.
    creation_time    timestamptz              NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- `last_updated` is set with each update to the row
    last_updated     timestamptz              NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- `visible_after` is essentially a visibility timeout. See the `Visibility Timeout` section below.
    visible_after    timestamptz              NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- `try_count` on every receive we increment `try_count` to indicate another attempt to process this message
    try_count        integer                  NOT NULL,
    -- We limit the message to 256MB, which is arbitrary but reasonable as an upper limit
    CHECK (length(pipeline_message) < plugin_work_queue.megabytes(256)),
    CHECK (last_updated >= creation_time)
)
;

CREATE INDEX IF NOT EXISTS execution_key_ix ON plugin_work_queue.generator_plugin_executions (execution_key);
CREATE INDEX IF NOT EXISTS creation_time_ix ON plugin_work_queue.generator_plugin_executions (creation_time);

CREATE INDEX IF NOT EXISTS execution_key_ix ON plugin_work_queue.analyzer_plugin_executions (execution_key);
CREATE INDEX IF NOT EXISTS creation_time_ix ON plugin_work_queue.analyzer_plugin_executions (creation_time);