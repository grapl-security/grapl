-- This column just duplicates data already stored in `plugin-registry`. 
ALTER TABLE plugin_work_queue.generator_plugin_executions
  DROP COLUMN tenant_id;
