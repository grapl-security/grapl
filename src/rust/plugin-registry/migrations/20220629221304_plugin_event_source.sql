-- Add migration script here

-- The event_source_id will be absent for analyzers, present for generators. In
-- the get_generators_for_event_source query we filter on both event_source_id
-- and plugin_type to ensure we won't erroneously return data for some 3rd
-- plugin type should we add one in the future. The event_source_plugin_type_ix
-- will also be useful at that future time if we need to implement a similar
-- query for that 3rd plugin type.
ALTER TABLE plugins
      ADD COLUMN event_source_id uuid DEFAULT NULL;

CREATE INDEX event_source_plugin_type_ix ON plugins (event_source_id, plugin_type);
