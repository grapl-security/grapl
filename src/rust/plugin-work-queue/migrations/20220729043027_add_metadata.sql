ALTER TABLE plugin_work_queue.generator_plugin_executions
    ADD COLUMN tenant_id uuid NOT NULL,
    ADD COLUMN trace_id uuid NOT NULL,
    ADD COLUMN event_source_id uuid NOT NULL;

ALTER TABLE plugin_work_queue.analyzer_plugin_executions
    ADD COLUMN trace_id uuid NOT NULL,
    ADD COLUMN event_source_id uuid NOT NULL;
