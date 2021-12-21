CREATE TYPE status_t AS ENUM ('enqueued', 'failed', 'processed');

CREATE SCHEMA plugin_work_queue;

CREATE FUNCTION megabytes(bytes integer) RETURNS integer AS
$$
BEGIN
    RETURN bytes * 1000 * 1000;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS plugin_work_queue.plugin_executions
(
    execution_key        bigserial   NOT NULL,
    tenant_id            uuid        NOT NULL,
    plugin_id            uuid        NOT NULL,
    pipeline_message     bytea       NOT NULL,
    execution_result     bytea,
    status               status_t    NOT NULL,
    creation_time        timestamptz NOT NULL,
    last_updated         timestamptz NOT NULL,
    visible_after        timestamptz,
    try_count            integer     NOT NULL,
    trace_id             uuid        NOT NULL,
    CHECK (length(pipeline_message) < megabytes(256)),
    CHECK (length(execution_result) < megabytes(256)),
    CHECK (last_updated >= creation_time)
)
    PARTITION BY RANGE (creation_time);

CREATE INDEX IF NOT EXISTS execution_key_ix ON plugin_work_queue.plugin_executions (execution_key);
CREATE INDEX IF NOT EXISTS creation_time_ix ON plugin_work_queue.plugin_executions (creation_time);

CREATE SCHEMA partman;
CREATE EXTENSION pg_partman WITH SCHEMA partman;


-- TODO: Check optimize_contraint
SELECT partman.create_parent(p_parent_table => 'plugin_work_queue.plugin_executions',
                             p_control => 'creation_time',
                             p_type => 'native',
                             p_interval=> 'daily',
                             p_premake => 30
           );

CREATE EXTENSION pg_cron;

UPDATE partman.part_config
SET infinite_time_partitions = true,
    retention                = '1 month',
    retention_keep_table= true
WHERE parent_table = 'plugin_work_queue.plugin_executions';

SELECT cron.schedule('@hourly', $$CALL partman.run_maintenance_proc()$$);